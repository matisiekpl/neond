use std::path::PathBuf;
use std::process::Stdio;
use std::str::FromStr;
use std::sync::Arc;

use neon_pageserver_api::models::{TimelineCreateRequest, TimelineCreateRequestMode};
use neon_utils::id::{TenantId, TimelineId};
use neon_utils::shard::TenantShardId;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use uuid::Uuid;

use crate::mgmt::compute::ComputeEndpointStatus;
use crate::mgmt::dto::branch_response::BranchResponse;
use crate::mgmt::dto::config::Config;
use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::dto::import_branch_request::ImportBranchRequest;
use crate::mgmt::repository::branch::BranchRepository;
use crate::mgmt::repository::project::ProjectRepository;
use crate::mgmt::service::branch::BranchService;
use crate::mgmt::service::endpoint::EndpointService;
use crate::mgmt::service::logs::{LogChannel, LogStream, LogsService};
use crate::mgmt::service::membership::MembershipService;
use crate::utils::password::generate_password;

pub struct ImportService {
    branch_repo: Arc<BranchRepository>,
    project_repo: Arc<ProjectRepository>,
    membership_service: Arc<MembershipService>,
    pageserver_client: Arc<neon_pageserver_client::mgmt_api::Client>,
    endpoint_service: Arc<EndpointService>,
    branch_service: Arc<BranchService>,
    logs_service: Arc<LogsService>,
    config: Config,
}

impl ImportService {
    pub fn new(
        branch_repo: Arc<BranchRepository>,
        project_repo: Arc<ProjectRepository>,
        membership_service: Arc<MembershipService>,
        pageserver_client: Arc<neon_pageserver_client::mgmt_api::Client>,
        endpoint_service: Arc<EndpointService>,
        branch_service: Arc<BranchService>,
        logs_service: Arc<LogsService>,
        config: Config,
    ) -> Self {
        Self {
            branch_repo,
            project_repo,
            membership_service,
            pageserver_client,
            endpoint_service,
            branch_service,
            logs_service,
            config,
        }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        project_id: Uuid,
        req: ImportBranchRequest,
    ) -> Result<BranchResponse> {
        BranchService::validate_branch_name(&req.name)?;

        tokio_postgres::Config::from_str(&req.source_connection_string)
            .map_err(|error| AppError::InvalidConnectionString {
                reason: error.to_string(),
            })?;

        self.membership_service
            .verify_membership(user_id, organization_id)
            .await?;

        let project = self
            .project_repo
            .find_by_id(project_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if project.organization_id != organization_id {
            return Err(AppError::NotFound);
        }

        if self
            .branch_repo
            .find_by_project_and_name(project_id, &req.name)
            .await?
            .is_some()
        {
            return Err(AppError::BranchNameAlreadyExists {
                name: req.name.clone(),
            });
        }

        let new_timeline_id = TimelineId::generate();
        let timeline_uuid = Uuid::from_str(new_timeline_id.to_string().as_str())
            .map_err(|_| AppError::TimelineIdInvalid {
                value: new_timeline_id.to_string(),
            })?;

        let tenant_id = TenantId::from_str(project_id.as_simple().to_string().as_str())
            .map_err(|_| AppError::TenantIdInvalid {
                value: project_id.to_string(),
            })?;

        self.pageserver_client
            .timeline_create(
                TenantShardId::unsharded(tenant_id),
                &TimelineCreateRequest {
                    new_timeline_id,
                    mode: TimelineCreateRequestMode::Bootstrap {
                        existing_initdb_timeline_id: None,
                        pg_version: None,
                    },
                },
            )
            .await
            .map_err(|error| AppError::BranchCreationFailed {
                reason: error.to_string(),
            })?;

        let id = Uuid::new_v4();
        let password = generate_password();
        let slug = self.branch_service.generate_unique_slug().await?;

        let branch = self
            .branch_repo
            .create_for_import(
                id,
                project_id,
                &req.name,
                timeline_uuid,
                &password,
                &slug,
            )
            .await?;

        self.logs_service.ingest(
            LogChannel::Import(branch.id),
            "Starting import".to_string(),
            LogStream::Stdout,
        );

        let endpoint = match self
            .endpoint_service
            .start(user_id, organization_id, project_id, branch.id)
            .await
        {
            Ok(endpoint) => endpoint,
            Err(error) => {
                let message = error.to_string();
                self.logs_service.ingest(
                    LogChannel::Import(branch.id),
                    format!("Endpoint failed to start: {}", message),
                    LogStream::Stderr,
                );
                if let Err(update_error) = self
                    .branch_repo
                    .update_import_status(branch.id, "failed", Some(&message))
                    .await
                {
                    tracing::error!(
                        "Failed to mark import as failed for branch {}: {}",
                        branch.id,
                        update_error
                    );
                }
                return Err(error);
            }
        };

        let branch_repo = Arc::clone(&self.branch_repo);
        let logs_service = Arc::clone(&self.logs_service);
        let pg_install_directory = self.config.pg_install_directory.clone();
        let source_connection_string = req.source_connection_string.clone();
        let target_port = endpoint.port;
        let target_password = branch.password.clone();
        let branch_id = branch.id;

        tokio::spawn(async move {
            match Self::run_import(
                Arc::clone(&logs_service),
                branch_id,
                pg_install_directory,
                source_connection_string,
                target_port,
                target_password,
            )
            .await
            {
                Ok(()) => {
                    logs_service.ingest(
                        LogChannel::Import(branch_id),
                        "Import finished successfully".to_string(),
                        LogStream::Stdout,
                    );
                    if let Err(error) = branch_repo
                        .update_import_status(branch_id, "ready", None)
                        .await
                    {
                        tracing::error!(
                            "Failed to mark import as ready for branch {}: {}",
                            branch_id,
                            error
                        );
                    }
                }
                Err(error) => {
                    let message = error.to_string();
                    logs_service.ingest(
                        LogChannel::Import(branch_id),
                        format!("Import failed: {}", message),
                        LogStream::Stderr,
                    );
                    tracing::error!(
                        "Import failed for branch {}: {}",
                        branch_id,
                        message
                    );
                    if let Err(update_error) = branch_repo
                        .update_import_status(branch_id, "failed", Some(&message))
                        .await
                    {
                        tracing::error!(
                            "Failed to mark import as failed for branch {}: {}",
                            branch_id,
                            update_error
                        );
                    }
                }
            }
        });

        let endpoint_info = self.endpoint_service.get_endpoint_info(branch.id).await;

        Ok(BranchResponse {
            id: branch.id,
            project_id: branch.project_id,
            name: branch.name.clone(),
            slug: branch.slug.clone(),
            parent_branch_id: branch.parent_branch_id,
            timeline_id: branch.timeline_id,
            ancestor_timeline_id: None,
            ancestor_lsn: None,
            endpoint_status: endpoint_info
                .clone()
                .map(|info| info.status)
                .unwrap_or(ComputeEndpointStatus::Stopped),
            remote_consistent_lsn_visible: Default::default(),
            last_record_lsn: Default::default(),
            current_logical_size: 0,
            connection_string: endpoint_info
                .clone()
                .map(|info| branch.get_connection_string(self.config.clone(), info.port)),
            pooler_connection_string: endpoint_info
                .and_then(|info| info.pooler_port)
                .map(|pooler_port| {
                    branch.get_pooler_connection_string(self.config.clone(), pooler_port)
                }),
            password: branch.password.clone(),
            created_at: branch.created_at,
            updated_at: branch.updated_at,
            import_status: branch.import_status.clone(),
            import_error: branch.import_error.clone(),
        })
    }

    async fn run_import(
        logs_service: Arc<LogsService>,
        branch_id: Uuid,
        pg_install_directory: PathBuf,
        source_connection_string: String,
        target_port: u16,
        target_password: String,
    ) -> Result<()> {
        let pg_dump_binary = pg_install_directory.join("vanilla_v17/bin/pg_dump");
        let pg_restore_binary = pg_install_directory.join("vanilla_v17/bin/pg_restore");
        let pg_lib = pg_install_directory.join("vanilla_v17/lib");

        let target_url = format!(
            "postgresql://postgres:{}@127.0.0.1:{}/postgres?sslmode=require",
            target_password, target_port,
        );

        logs_service.ingest(
            LogChannel::Import(branch_id),
            format!(
                "Running {} | {} (target port {})",
                pg_dump_binary.display(),
                pg_restore_binary.display(),
                target_port,
            ),
            LogStream::Stdout,
        );

        let mut pg_dump = Command::new(&pg_dump_binary)
            .env("DYLD_LIBRARY_PATH", &pg_lib)
            .env("LD_LIBRARY_PATH", &pg_lib)
            .arg("-Fc")
            .arg("--no-owner")
            .arg("--no-acl")
            .arg("--no-publications")
            .arg("--no-subscriptions")
            .arg("--exclude-schema=neon")
            .arg("--exclude-schema=neon_migration")
            .arg("--exclude-extension=neon")
            .arg("-d")
            .arg(&source_connection_string)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|error| AppError::BranchImportFailed {
                reason: format!("spawn pg_dump: {}", error),
            })?;

        let mut pg_restore = Command::new(&pg_restore_binary)
            .env("DYLD_LIBRARY_PATH", &pg_lib)
            .env("LD_LIBRARY_PATH", &pg_lib)
            .arg("--no-owner")
            .arg("--no-acl")
            .arg("-d")
            .arg(&target_url)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|error| AppError::BranchImportFailed {
                reason: format!("spawn pg_restore: {}", error),
            })?;

        let mut dump_stdout = pg_dump.stdout.take().ok_or_else(|| {
            AppError::BranchImportFailed {
                reason: "pg_dump stdout unavailable".into(),
            }
        })?;
        let mut restore_stdin = pg_restore.stdin.take().ok_or_else(|| {
            AppError::BranchImportFailed {
                reason: "pg_restore stdin unavailable".into(),
            }
        })?;

        tokio::spawn(async move {
            if let Err(error) = tokio::io::copy(&mut dump_stdout, &mut restore_stdin).await {
                tracing::error!("Failed to pipe pg_dump to pg_restore: {}", error);
            }
            drop(restore_stdin);
        });

        if let Some(stderr) = pg_dump.stderr.take() {
            tokio::spawn(Self::stream_lines(
                Arc::clone(&logs_service),
                branch_id,
                "pg_dump",
                stderr,
                LogStream::Stderr,
            ));
        }
        if let Some(stderr) = pg_restore.stderr.take() {
            tokio::spawn(Self::stream_lines(
                Arc::clone(&logs_service),
                branch_id,
                "pg_restore",
                stderr,
                LogStream::Stderr,
            ));
        }
        if let Some(stdout) = pg_restore.stdout.take() {
            tokio::spawn(Self::stream_lines(
                Arc::clone(&logs_service),
                branch_id,
                "pg_restore",
                stdout,
                LogStream::Stdout,
            ));
        }

        let dump_status = pg_dump.wait().await.map_err(|error| {
            AppError::BranchImportFailed {
                reason: format!("wait pg_dump: {}", error),
            }
        })?;
        let restore_status = pg_restore.wait().await.map_err(|error| {
            AppError::BranchImportFailed {
                reason: format!("wait pg_restore: {}", error),
            }
        })?;

        if !dump_status.success() {
            return Err(AppError::BranchImportFailed {
                reason: format!("pg_dump exited with {}", dump_status),
            });
        }
        if !restore_status.success() {
            return Err(AppError::BranchImportFailed {
                reason: format!("pg_restore exited with {}", restore_status),
            });
        }

        Ok(())
    }

    async fn stream_lines<R>(
        logs_service: Arc<LogsService>,
        branch_id: Uuid,
        component: &'static str,
        reader: R,
        stream: LogStream,
    )
    where
        R: tokio::io::AsyncRead + Unpin + Send + 'static,
    {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            logs_service.ingest(
                LogChannel::Import(branch_id),
                format!("[{}] {}", component, line),
                stream,
            );
        }
    }
}
