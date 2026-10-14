use std::collections::{HashMap, HashSet, VecDeque};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStderr, ChildStdout, Command};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use uuid::Uuid;

use crate::mgmt::compute::ComputeEndpointStatus;
use crate::mgmt::dto::config::Config;
use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::dto::export_branch_request::{ExportBranchRequest, ExportFormat};
use crate::mgmt::dto::export_download_token_response::ExportDownloadTokenResponse;
use crate::mgmt::dto::export_status_response::{ExportState, ExportStatusResponse};
use crate::mgmt::model::branch::{Branch, ImportStatus};
use crate::mgmt::model::project::{PgVersion, Project};
use crate::mgmt::repository::branch::BranchRepository;
use crate::mgmt::repository::project::ProjectRepository;
use crate::mgmt::service::endpoint::EndpointService;
use crate::mgmt::service::logs::{LogChannel, LogStream, LogsService};
use crate::mgmt::service::membership::MembershipService;

const MAX_CONCURRENT_EXPORTS: usize = 2;
const EXPORT_RETENTION_SECS: i64 = 60 * 60;
const DOWNLOAD_TOKEN_TTL_SECS: i64 = 60;
const DOWNLOAD_TOKEN_PURPOSE: &str = "export_download";
const DISK_RESERVE_BYTES: u64 = 1 << 30;
const DISK_ABORT_BYTES: u64 = 512 << 20;
const DISK_CHECK_INTERVAL: Duration = Duration::from_secs(5);
const CLEANUP_INTERVAL: Duration = Duration::from_secs(60);
const COMPUTE_READY_TIMEOUT: Duration = Duration::from_secs(120);
const STDERR_TAIL_LINES: usize = 20;
const PROGRESS_REPORT_BYTES: u64 = 1 << 20;
const NEON_INTERNAL_SCHEMAS: &[&str] = &["neon", "neon_migration"];
const NEON_INTERNAL_EXTENSIONS: &[&str] = &["neon"];
/// Created by compute_ctl on start, dropped by neond a few seconds later.
const NEON_INTERNAL_TABLES: &[&str] = &["public.health_check"];
const ALL_PG_VERSIONS: &[PgVersion] = &[
    PgVersion::V14,
    PgVersion::V15,
    PgVersion::V16,
    PgVersion::V17,
];

const TABLE_SIZES_SQL: &str = "
    SELECT n.nspname::text, c.relname::text, pg_table_size(c.oid)
    FROM pg_class c
    JOIN pg_namespace n ON n.oid = c.relnamespace
    WHERE c.relkind = 'r'
      AND n.nspname NOT IN ('pg_catalog', 'information_schema', 'neon', 'neon_migration')
      AND n.nspname !~ '^pg_(toast|temp_)'
      AND NOT (n.nspname = 'public' AND c.relname = 'health_check')
      AND NOT EXISTS (
          SELECT 1 FROM pg_depend d
          WHERE d.classid = 'pg_class'::regclass AND d.objid = c.oid AND d.deptype = 'e'
      )";

#[derive(Serialize, Deserialize)]
struct DownloadClaims {
    sub: Uuid,
    branch_id: Uuid,
    export_id: Uuid,
    jti: Uuid,
    purpose: String,
    exp: usize,
}

struct ExportJob {
    status: ExportStatusResponse,
    path: PathBuf,
    cancel: CancellationToken,
    used_download_tokens: HashSet<Uuid>,
}

type Jobs = Arc<Mutex<HashMap<Uuid, ExportJob>>>;

pub struct ExportDownload {
    pub file: tokio::fs::File,
    pub length: u64,
    pub file_name: String,
    pub content_type: &'static str,
}

pub struct ExportService {
    branch_repo: Arc<BranchRepository>,
    project_repo: Arc<ProjectRepository>,
    membership_service: Arc<MembershipService>,
    endpoint_service: Arc<EndpointService>,
    logs_service: Arc<LogsService>,
    config: Config,
    shutdown_token: CancellationToken,
    tracker: TaskTracker,
    jobs: Jobs,
    exports_directory: PathBuf,
    download_key: Vec<u8>,
}

impl ExportService {
    pub fn new(
        branch_repo: Arc<BranchRepository>,
        project_repo: Arc<ProjectRepository>,
        membership_service: Arc<MembershipService>,
        endpoint_service: Arc<EndpointService>,
        logs_service: Arc<LogsService>,
        config: Config,
        shutdown_token: CancellationToken,
    ) -> Self {
        let exports_directory = config.daemon_directory.join("exports");
        let download_key = derive_download_key(&config.server_secret);
        Self {
            branch_repo,
            project_repo,
            membership_service,
            endpoint_service,
            logs_service,
            config,
            shutdown_token,
            tracker: TaskTracker::new(),
            jobs: Arc::new(Mutex::new(HashMap::new())),
            exports_directory,
            download_key,
        }
    }

    pub async fn start(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
        req: ExportBranchRequest,
    ) -> Result<ExportStatusResponse> {
        validate_database_name(&req.database)?;

        let (project, branch) = self
            .resolve(user_id, organization_id, project_id, branch_id)
            .await?;

        if branch.import_status.as_deref() == Some(ImportStatus::Importing.as_str()) {
            return Err(AppError::ExportBranchImporting);
        }

        let (pg_dump, pg_lib) = self.config.pg_tool(&project.pg_version, "pg_dump");
        if !pg_dump.exists() {
            return Err(AppError::ExportToolMissing {
                path: pg_dump.display().to_string(),
            });
        }

        let export_id = Uuid::new_v4();
        let started_at = Utc::now();
        let path = self
            .exports_directory
            .join(format!("{export_id}.{}", req.format.extension()));
        let file_name = export_file_name(
            &project.name,
            &branch.name,
            &req.database,
            req.format,
            started_at,
        );
        let cancel = CancellationToken::new();

        let (status, previous_path) = {
            let mut jobs = self.jobs.lock().unwrap();
            if jobs
                .get(&branch_id)
                .is_some_and(|job| job.status.state.is_active())
            {
                return Err(AppError::ExportAlreadyRunning);
            }
            let running = jobs
                .values()
                .filter(|job| job.status.state.is_active())
                .count();
            if running >= MAX_CONCURRENT_EXPORTS {
                return Err(AppError::ExportCapacityExceeded);
            }
            let status = ExportStatusResponse {
                export_id: Some(export_id),
                state: ExportState::Preparing,
                format: Some(req.format),
                database: Some(req.database.clone()),
                pg_version: Some(project.pg_version.clone()),
                started_at: Some(started_at),
                file_name: Some(file_name),
                ..Default::default()
            };
            let previous = jobs.insert(
                branch_id,
                ExportJob {
                    status: status.clone(),
                    path: path.clone(),
                    cancel: cancel.clone(),
                    used_download_tokens: HashSet::new(),
                },
            );
            (status, previous.map(|job| job.path))
        };

        if let Some(previous_path) = previous_path {
            remove_file_if_exists(&previous_path).await;
        }

        tracing::info!(
            "Starting {} export {} of database {:?} on branch {}",
            req.format.pg_dump_format(),
            export_id,
            req.database,
            branch_id
        );

        let job = ExportRun {
            jobs: Arc::clone(&self.jobs),
            endpoint_service: Arc::clone(&self.endpoint_service),
            logs_service: Arc::clone(&self.logs_service),
            user_id,
            organization_id,
            project_id,
            branch_id,
            export_id,
            pg_version: project.pg_version,
            pg_dump,
            pg_lib,
            exports_directory: self.exports_directory.clone(),
            path,
            database: req.database,
            format: req.format,
            no_owner: req.no_owner,
            no_privileges: req.no_privileges,
            cancel,
            shutdown_token: self.shutdown_token.clone(),
        };
        self.tracker.spawn(job.run());

        Ok(status)
    }

    pub async fn status(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
    ) -> Result<ExportStatusResponse> {
        self.resolve(user_id, organization_id, project_id, branch_id)
            .await?;
        Ok(self
            .jobs
            .lock()
            .unwrap()
            .get(&branch_id)
            .map(|job| job.status.clone())
            .unwrap_or_default())
    }

    pub async fn cancel(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
    ) -> Result<ExportStatusResponse> {
        self.resolve(user_id, organization_id, project_id, branch_id)
            .await?;

        let removed_path = {
            let mut jobs = self.jobs.lock().unwrap();
            let Some(job) = jobs.get(&branch_id) else {
                return Ok(ExportStatusResponse::default());
            };
            if job.status.state.is_active() {
                job.cancel.cancel();
                return Ok(job.status.clone());
            }
            jobs.remove(&branch_id).map(|job| job.path)
        };

        if let Some(path) = removed_path {
            remove_file_if_exists(&path).await;
        }
        Ok(ExportStatusResponse::default())
    }

    pub async fn discard_for_branch(&self, branch_id: Uuid) {
        let removed = self.jobs.lock().unwrap().remove(&branch_id);
        if let Some(job) = removed {
            job.cancel.cancel();
            if !job.status.state.is_active() {
                remove_file_if_exists(&job.path).await;
            }
        }
    }

    pub async fn create_download_token(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
    ) -> Result<ExportDownloadTokenResponse> {
        self.resolve(user_id, organization_id, project_id, branch_id)
            .await?;

        let export_id = self
            .jobs
            .lock()
            .unwrap()
            .get(&branch_id)
            .filter(|job| job.status.state == ExportState::Completed)
            .and_then(|job| job.status.export_id)
            .ok_or(AppError::ExportNotReady)?;

        let expires_at = Utc::now() + chrono::Duration::seconds(DOWNLOAD_TOKEN_TTL_SECS);
        let claims = DownloadClaims {
            sub: user_id,
            branch_id,
            export_id,
            jti: Uuid::new_v4(),
            purpose: DOWNLOAD_TOKEN_PURPOSE.to_string(),
            exp: expires_at.timestamp() as usize,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&self.download_key),
        )
        .map_err(|error| AppError::TokenGenerationFailed {
            reason: error.to_string(),
        })?;

        Ok(ExportDownloadTokenResponse { token, expires_at })
    }

    pub async fn open_download(
        &self,
        organization_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
        token: &str,
    ) -> Result<ExportDownload> {
        let mut validation = Validation::default();
        validation.leeway = 5;
        let claims = decode::<DownloadClaims>(
            token,
            &DecodingKey::from_secret(&self.download_key),
            &validation,
        )
        .map_err(|_| AppError::Unauthorized)?
        .claims;

        if claims.purpose != DOWNLOAD_TOKEN_PURPOSE || claims.branch_id != branch_id {
            return Err(AppError::Unauthorized);
        }

        self.resolve(claims.sub, organization_id, project_id, branch_id)
            .await?;

        let (path, file_name, format) = {
            let mut jobs = self.jobs.lock().unwrap();
            let job = jobs.get_mut(&branch_id).ok_or(AppError::Unauthorized)?;
            if job.status.export_id != Some(claims.export_id) {
                return Err(AppError::Unauthorized);
            }
            if job.status.state != ExportState::Completed {
                return Err(AppError::ExportNotReady);
            }
            if !job.used_download_tokens.insert(claims.jti) {
                return Err(AppError::Unauthorized);
            }
            (
                job.path.clone(),
                job.status.file_name.clone().unwrap_or_default(),
                job.status.format.unwrap_or(ExportFormat::Plain),
            )
        };

        let file = tokio::fs::File::open(&path)
            .await
            .map_err(|_| AppError::NotFound)?;
        let length = file.metadata().await?.len();

        tracing::info!(
            "User {} downloads export {} of branch {} ({} bytes)",
            claims.sub,
            claims.export_id,
            branch_id,
            length
        );

        Ok(ExportDownload {
            file,
            length,
            file_name,
            content_type: format.content_type(),
        })
    }

    pub async fn authorize(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
    ) -> Result<()> {
        self.resolve(user_id, organization_id, project_id, branch_id)
            .await
            .map(|_| ())
    }

    pub async fn reconcile_on_start(&self) {
        match tokio::fs::remove_dir_all(&self.exports_directory).await {
            Ok(()) => tracing::info!("Removed export files left from the previous run"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => tracing::warn!(
                "Failed to remove export files in {}: {}",
                self.exports_directory.display(),
                error
            ),
        }

        let missing: Vec<String> = ALL_PG_VERSIONS
            .iter()
            .filter(|version| !self.config.pg_tool(version, "pg_dump").0.exists())
            .map(|version| version.to_string())
            .collect();
        if !missing.is_empty() {
            tracing::warn!(
                "pg_dump is missing for Postgres {}; branches of projects on these versions cannot be exported",
                missing.join(", ")
            );
        }
    }

    pub async fn run_cleanup_loop(self: Arc<Self>, shutdown_token: CancellationToken) {
        let mut interval = tokio::time::interval(CLEANUP_INTERVAL);
        loop {
            tokio::select! {
                _ = shutdown_token.cancelled() => break,
                _ = interval.tick() => self.cleanup().await,
            }
        }
    }

    async fn cleanup(&self) {
        let branch_ids: Vec<Uuid> = self.jobs.lock().unwrap().keys().copied().collect();
        let mut deleted_branches = HashSet::new();
        for branch_id in branch_ids {
            if let Ok(None) = self.branch_repo.find_by_id(branch_id).await {
                deleted_branches.insert(branch_id);
            }
        }

        let now = Utc::now();
        let mut expired_paths = Vec::new();
        self.jobs.lock().unwrap().retain(|branch_id, job| {
            let active = job.status.state.is_active();
            let branch_deleted = deleted_branches.contains(branch_id);
            let expired = !active && job.status.expires_at.is_some_and(|at| at <= now);
            if !branch_deleted && !expired {
                return true;
            }
            job.cancel.cancel();
            if !active {
                expired_paths.push(job.path.clone());
            }
            false
        });

        for path in expired_paths {
            remove_file_if_exists(&path).await;
        }
    }

    pub async fn shutdown(&self) {
        self.tracker.close();
        self.tracker.wait().await;
        let _ = tokio::fs::remove_dir_all(&self.exports_directory).await;
    }

    async fn resolve(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
    ) -> Result<(Project, Branch)> {
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

        let branch = self
            .branch_repo
            .find_by_id(branch_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if branch.project_id != project_id {
            return Err(AppError::NotFound);
        }

        Ok((project, branch))
    }
}

enum ExportError {
    Cancelled,
    Failed(String),
}

struct DumpPlan {
    table_sizes: HashMap<String, u64>,
    total_bytes: u64,
    extensions: Vec<String>,
    has_neon_extension: bool,
}

struct ExportRun {
    jobs: Jobs,
    endpoint_service: Arc<EndpointService>,
    logs_service: Arc<LogsService>,
    user_id: Uuid,
    organization_id: Uuid,
    project_id: Uuid,
    branch_id: Uuid,
    export_id: Uuid,
    pg_version: PgVersion,
    pg_dump: PathBuf,
    pg_lib: PathBuf,
    exports_directory: PathBuf,
    path: PathBuf,
    database: String,
    format: ExportFormat,
    no_owner: bool,
    no_privileges: bool,
    cancel: CancellationToken,
    shutdown_token: CancellationToken,
}

impl ExportRun {
    async fn run(self) {
        match self.execute().await {
            Ok(bytes_written) => {
                let finished_at = Utc::now();
                let still_tracked = self.update(|status| {
                    status.state = ExportState::Completed;
                    status.progress = 1.0;
                    status.tables_done = status.tables_total;
                    status.current_table = None;
                    status.bytes_written = bytes_written;
                    status.finished_at = Some(finished_at);
                    status.expires_at =
                        Some(finished_at + chrono::Duration::seconds(EXPORT_RETENTION_SECS));
                });
                if !still_tracked {
                    remove_file_if_exists(&self.path).await;
                    return;
                }
                self.log(
                    format!("Export finished: {} bytes", bytes_written),
                    LogStream::Stdout,
                );
                tracing::info!(
                    "Export {} of branch {} finished: {} bytes",
                    self.export_id,
                    self.branch_id,
                    bytes_written
                );
            }
            Err(error) => {
                remove_file_if_exists(&self.path).await;
                let (state, message) = match error {
                    ExportError::Cancelled => (ExportState::Cancelled, None),
                    ExportError::Failed(message) => (ExportState::Failed, Some(message)),
                };
                match &message {
                    Some(message) => {
                        self.log(format!("Export failed: {}", message), LogStream::Stderr);
                        tracing::warn!(
                            "Export {} of branch {} failed: {}",
                            self.export_id,
                            self.branch_id,
                            message
                        );
                    }
                    None => self.log("Export cancelled".to_string(), LogStream::Stderr),
                }
                self.update(|status| {
                    status.state = state;
                    status.error = message;
                    status.current_table = None;
                    status.finished_at = Some(Utc::now());
                });
            }
        }
    }

    async fn execute(&self) -> std::result::Result<u64, ExportError> {
        self.log(
            format!(
                "Starting {} export of database \"{}\" with pg_dump {}",
                self.format.pg_dump_format(),
                self.database,
                self.pg_version
            ),
            LogStream::Stdout,
        );

        let pg_dump_version = self.pg_dump_version().await?;
        self.update(|status| status.pg_dump_version = Some(pg_dump_version.clone()));

        let port = self.ensure_compute().await?;
        self.check_interrupted()?;

        let plan = self.interruptible(self.inspect_database(port)).await?;
        self.log(
            format!(
                "{} tables to dump, {} bytes in total",
                plan.table_sizes.len(),
                plan.total_bytes
            ),
            LogStream::Stdout,
        );
        self.update(|status| {
            status.tables_total = plan.table_sizes.len() as u64;
            status.bytes_total_estimate = plan.total_bytes;
        });

        self.check_disk_space(plan.total_bytes)?;

        self.update(|status| status.state = ExportState::Dumping);
        self.dump(port, &plan).await
    }

    async fn pg_dump_version(&self) -> std::result::Result<String, ExportError> {
        let output = Command::new(&self.pg_dump)
            .env_clear()
            .env("LD_LIBRARY_PATH", &self.pg_lib)
            .env("DYLD_LIBRARY_PATH", &self.pg_lib)
            .env("LC_ALL", "C")
            .arg("--version")
            .stdin(Stdio::null())
            .kill_on_drop(true)
            .output()
            .await
            .map_err(|error| ExportError::Failed(format!("could not run pg_dump: {error}")))?;
        if !output.status.success() {
            return Err(ExportError::Failed(format!(
                "pg_dump --version exited with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        Ok(parse_pg_dump_version(&String::from_utf8_lossy(&output.stdout)))
    }

    async fn ensure_compute(&self) -> std::result::Result<u16, ExportError> {
        let deadline = Instant::now() + COMPUTE_READY_TIMEOUT;
        loop {
            match self.endpoint_service.get_endpoint_info(self.branch_id).await {
                Some(info) if info.status == ComputeEndpointStatus::Running => {
                    return Ok(info.port);
                }
                Some(info)
                    if matches!(
                        info.status,
                        ComputeEndpointStatus::Starting | ComputeEndpointStatus::Stopping
                    ) => {}
                _ => {
                    self.log("Starting the branch's compute".to_string(), LogStream::Stdout);
                    match self
                        .endpoint_service
                        .start(
                            self.user_id,
                            self.organization_id,
                            self.project_id,
                            self.branch_id,
                        )
                        .await
                    {
                        Ok(endpoint) => return Ok(endpoint.port),
                        Err(AppError::Conflict(_)) => {}
                        Err(error) => {
                            return Err(ExportError::Failed(format!(
                                "could not start the branch's compute: {error}"
                            )));
                        }
                    }
                }
            }
            if Instant::now() >= deadline {
                return Err(ExportError::Failed(
                    "timed out waiting for the branch's compute to start".to_string(),
                ));
            }
            self.check_interrupted()?;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    async fn inspect_database(&self, port: u16) -> std::result::Result<DumpPlan, ExportError> {
        let admin = connect(port, "postgres").await?;
        let database = admin
            .query_opt(
                "SELECT datallowconn, datistemplate FROM pg_database WHERE datname = $1",
                &[&self.database],
            )
            .await
            .map_err(|error| ExportError::Failed(format!("could not look up the database: {error}")))?;
        match database {
            None => {
                return Err(ExportError::Failed(format!(
                    "database \"{}\" does not exist",
                    self.database
                )));
            }
            Some(row) => {
                let allows_connections: bool = row.get(0);
                let is_template: bool = row.get(1);
                if !allows_connections || is_template {
                    return Err(ExportError::Failed(format!(
                        "database \"{}\" is a template or does not allow connections",
                        self.database
                    )));
                }
            }
        }
        drop(admin);

        let client = connect(port, &self.database).await?;
        let rows = client
            .query(TABLE_SIZES_SQL, &[])
            .await
            .map_err(|error| ExportError::Failed(format!("could not list tables: {error}")))?;
        let mut table_sizes = HashMap::new();
        let mut total_bytes = 0u64;
        for row in rows {
            let schema: String = row.get(0);
            let table: String = row.get(1);
            let size = row.get::<_, i64>(2).max(0) as u64;
            total_bytes += size;
            table_sizes.insert(format!("{schema}.{table}"), size);
        }

        let rows = client
            .query("SELECT extname::text FROM pg_extension ORDER BY 1", &[])
            .await
            .map_err(|error| ExportError::Failed(format!("could not list extensions: {error}")))?;
        let mut extensions = Vec::new();
        let mut has_neon_extension = false;
        for row in rows {
            let name: String = row.get(0);
            if NEON_INTERNAL_EXTENSIONS.contains(&name.as_str()) {
                has_neon_extension = true;
            } else {
                extensions.push(name);
            }
        }

        Ok(DumpPlan {
            table_sizes,
            total_bytes,
            extensions,
            has_neon_extension,
        })
    }

    fn check_disk_space(&self, estimate: u64) -> std::result::Result<(), ExportError> {
        let directory = self.exports_directory.parent().unwrap_or(self.exports_directory.as_path());
        let available = fs2::statvfs(directory)
            .map_err(|error| ExportError::Failed(format!("could not check free disk space: {error}")))?
            .available_space();
        let required = estimate.saturating_add(DISK_RESERVE_BYTES);
        if available < required {
            return Err(ExportError::Failed(format!(
                "not enough free disk space on the server: the export needs about {} bytes, {} are available",
                required, available
            )));
        }
        Ok(())
    }

    async fn dump(&self, port: u16, plan: &DumpPlan) -> std::result::Result<u64, ExportError> {
        let file = self.create_file().await?;

        let mut command = Command::new(&self.pg_dump);
        command
            .env_clear()
            .env("LD_LIBRARY_PATH", &self.pg_lib)
            .env("DYLD_LIBRARY_PATH", &self.pg_lib)
            .env("LC_ALL", "C")
            .env("PGHOST", "127.0.0.1")
            .env("PGPORT", port.to_string())
            .env("PGUSER", "cloud_admin")
            .env("PGDATABASE", &self.database)
            .env("PGSSLMODE", "disable")
            .env("PGAPPNAME", "neond-export")
            .env("PGCONNECT_TIMEOUT", "30")
            .arg("--verbose")
            .arg("--no-password")
            .arg(format!("--format={}", self.format.pg_dump_format()));
        if self.no_owner {
            command.arg("--no-owner");
        }
        if self.no_privileges {
            command.arg("--no-privileges");
        }
        for schema in NEON_INTERNAL_SCHEMAS {
            command.arg(format!("--exclude-schema={schema}"));
        }
        for table in NEON_INTERNAL_TABLES {
            command.arg(format!("--exclude-table={table}"));
        }
        // --exclude-extension needs pg_dump 17.
        if !plan.extensions.is_empty() {
            for extension in &plan.extensions {
                command.arg(format!("--extension={}", quote_pattern(extension)));
            }
        } else if plan.has_neon_extension {
            if matches!(self.pg_version, PgVersion::V17) {
                command.arg("--exclude-extension=neon");
            } else {
                self.log(
                    "The database has no extension besides neon; the dump will contain CREATE EXTENSION neon".to_string(),
                    LogStream::Stderr,
                );
            }
        }
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = command
            .spawn()
            .map_err(|error| ExportError::Failed(format!("could not start pg_dump: {error}")))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ExportError::Failed("pg_dump stdout unavailable".to_string()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| ExportError::Failed("pg_dump stderr unavailable".to_string()))?;

        enum Outcome {
            Finished(std::io::Result<u64>, Vec<String>, std::io::Result<std::process::ExitStatus>),
            Interrupted(ExportError),
        }

        let outcome = {
            let work = async {
                let (written, stderr_tail, exit_status) = tokio::join!(
                    self.copy_output(stdout, file),
                    self.follow_stderr(stderr, plan),
                    child.wait(),
                );
                Outcome::Finished(written, stderr_tail, exit_status)
            };
            tokio::select! {
                biased;
                _ = self.cancel.cancelled() => Outcome::Interrupted(ExportError::Cancelled),
                _ = self.shutdown_token.cancelled() => Outcome::Interrupted(ExportError::Failed(
                    "aborted: neond is shutting down".to_string(),
                )),
                reason = self.watch_disk() => Outcome::Interrupted(ExportError::Failed(reason)),
                outcome = work => outcome,
            }
        };

        match outcome {
            Outcome::Interrupted(error) => {
                let _ = child.kill().await;
                Err(error)
            }
            Outcome::Finished(written, stderr_tail, exit_status) => {
                // pg_dump dies of SIGPIPE when the write fails.
                let written = written.map_err(|error| {
                    ExportError::Failed(format!("writing the export file failed: {error}"))
                })?;
                let exit_status = exit_status
                    .map_err(|error| ExportError::Failed(format!("waiting for pg_dump: {error}")))?;
                if !exit_status.success() {
                    return Err(ExportError::Failed(format!(
                        "pg_dump exited with {}: {}",
                        exit_status,
                        stderr_tail.join("\n")
                    )));
                }
                Ok(written)
            }
        }
    }

    async fn create_file(&self) -> std::result::Result<tokio::fs::File, ExportError> {
        let failed =
            |error: std::io::Error| ExportError::Failed(format!("could not create the export file: {error}"));
        tokio::fs::create_dir_all(&self.exports_directory)
            .await
            .map_err(failed)?;
        tokio::fs::set_permissions(
            &self.exports_directory,
            std::fs::Permissions::from_mode(0o700),
        )
        .await
        .map_err(failed)?;
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&self.path)
            .map_err(failed)?;
        Ok(tokio::fs::File::from_std(file))
    }

    async fn copy_output(
        &self,
        mut stdout: ChildStdout,
        file: tokio::fs::File,
    ) -> std::io::Result<u64> {
        let mut writer = tokio::io::BufWriter::with_capacity(1 << 20, file);
        let mut buffer = vec![0u8; 256 * 1024];
        let mut written = 0u64;
        let mut reported = 0u64;
        loop {
            let read = stdout.read(&mut buffer).await?;
            if read == 0 {
                break;
            }
            writer.write_all(&buffer[..read]).await?;
            written += read as u64;
            if written - reported >= PROGRESS_REPORT_BYTES {
                reported = written;
                self.update(|status| status.bytes_written = written);
            }
        }
        self.update(|status| {
            status.bytes_written = written;
            status.state = ExportState::Finalizing;
        });
        writer.flush().await?;
        writer.into_inner().sync_all().await?;
        Ok(written)
    }

    async fn follow_stderr(&self, stderr: ChildStderr, plan: &DumpPlan) -> Vec<String> {
        let tables_total = plan.table_sizes.len() as u64;
        let mut tail = VecDeque::with_capacity(STDERR_TAIL_LINES);
        let mut done_bytes = 0u64;
        let mut tables_done = 0u64;
        let mut current_size: Option<u64> = None;
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Some(table) = parse_dumped_table(&line) {
                if let Some(size) = current_size.take() {
                    done_bytes += size;
                    tables_done += 1;
                }
                current_size = Some(plan.table_sizes.get(table).copied().unwrap_or(0));
                let progress = estimate_progress(done_bytes, plan.total_bytes, tables_done, tables_total);
                let table = table.to_string();
                self.update(|status| {
                    status.current_table = Some(table);
                    status.tables_done = tables_done.min(tables_total);
                    status.progress = progress;
                });
            }
            if tail.len() == STDERR_TAIL_LINES {
                tail.pop_front();
            }
            tail.push_back(line.clone());
            self.log(format!("[pg_dump] {line}"), LogStream::Stderr);
        }
        tail.into()
    }

    async fn watch_disk(&self) -> String {
        let directory = self.exports_directory.clone();
        let mut interval = tokio::time::interval(DISK_CHECK_INTERVAL);
        loop {
            interval.tick().await;
            if let Ok(stats) = fs2::statvfs(&directory) {
                if stats.available_space() < DISK_ABORT_BYTES {
                    return format!(
                        "aborted: less than {} MiB of disk space left on the server",
                        DISK_ABORT_BYTES >> 20
                    );
                }
            }
        }
    }

    async fn interruptible<T>(
        &self,
        future: impl Future<Output = std::result::Result<T, ExportError>>,
    ) -> std::result::Result<T, ExportError> {
        tokio::select! {
            biased;
            _ = self.cancel.cancelled() => Err(ExportError::Cancelled),
            _ = self.shutdown_token.cancelled() => Err(ExportError::Failed(
                "aborted: neond is shutting down".to_string(),
            )),
            result = future => result,
        }
    }

    fn check_interrupted(&self) -> std::result::Result<(), ExportError> {
        if self.cancel.is_cancelled() {
            return Err(ExportError::Cancelled);
        }
        if self.shutdown_token.is_cancelled() {
            return Err(ExportError::Failed(
                "aborted: neond is shutting down".to_string(),
            ));
        }
        Ok(())
    }

    fn update(&self, change: impl FnOnce(&mut ExportStatusResponse)) -> bool {
        let mut jobs = self.jobs.lock().unwrap();
        match jobs.get_mut(&self.branch_id) {
            Some(job) if job.status.export_id == Some(self.export_id) => {
                change(&mut job.status);
                true
            }
            _ => false,
        }
    }

    fn log(&self, message: String, stream: LogStream) {
        self.logs_service
            .ingest(LogChannel::Export(self.branch_id), message, stream);
    }
}

async fn connect(port: u16, database: &str) -> std::result::Result<tokio_postgres::Client, ExportError> {
    let mut config = tokio_postgres::Config::new();
    config
        .host("127.0.0.1")
        .port(port)
        .user("cloud_admin")
        .dbname(database)
        .application_name("neond-export")
        .connect_timeout(Duration::from_secs(30));
    let (client, connection) = config
        .connect(tokio_postgres::NoTls)
        .await
        .map_err(|error| {
            ExportError::Failed(format!(
                "could not connect to database \"{database}\": {error}"
            ))
        })?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            tracing::debug!("Export inspection connection closed: {}", error);
        }
    });
    Ok(client)
}

fn derive_download_key(server_secret: &str) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(server_secret.as_bytes())
        .expect("HMAC accepts keys of any length");
    mac.update(b"neond export download token v1");
    mac.finalize().into_bytes().to_vec()
}

fn validate_database_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 63 {
        return Err(AppError::ExportDatabaseInvalid {
            reason: "the name must be between 1 and 63 bytes long".to_string(),
        });
    }
    // libpq parses such names as connection strings.
    if name.chars().any(|c| c.is_control() || c == '=')
        || name.starts_with("postgres://")
        || name.starts_with("postgresql://")
    {
        return Err(AppError::ExportDatabaseInvalid {
            reason: "the name contains unsupported characters".to_string(),
        });
    }
    Ok(())
}

fn quote_pattern(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// `pg_dump (PostgreSQL) 15.13 ()` → `15.13`
fn parse_pg_dump_version(output: &str) -> String {
    output
        .split_whitespace()
        .find(|token| token.starts_with(|c: char| c.is_ascii_digit()))
        .unwrap_or_default()
        .to_string()
}

fn parse_dumped_table(line: &str) -> Option<&str> {
    const MARKER: &str = "dumping contents of table \"";
    let start = line.find(MARKER)? + MARKER.len();
    let rest = &line[start..];
    let end = rest.rfind('"')?;
    Some(&rest[..end])
}

fn estimate_progress(done_bytes: u64, total_bytes: u64, tables_done: u64, tables_total: u64) -> f64 {
    let progress = if total_bytes > 0 {
        done_bytes as f64 / total_bytes as f64
    } else if tables_total > 0 {
        tables_done as f64 / tables_total as f64
    } else {
        0.0
    };
    progress.clamp(0.0, 0.99)
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug: String = slug.chars().take(40).collect();
    slug.trim_end_matches('-').to_string()
}

fn export_file_name(
    project_name: &str,
    branch_name: &str,
    database: &str,
    format: ExportFormat,
    at: DateTime<Utc>,
) -> String {
    let mut parts: Vec<String> = [project_name, branch_name]
        .iter()
        .map(|value| slugify(value))
        .filter(|slug| !slug.is_empty())
        .collect();
    if database != "postgres" {
        let slug = slugify(database);
        if !slug.is_empty() {
            parts.push(slug);
        }
    }
    if parts.is_empty() {
        parts.push("export".to_string());
    }
    parts.push(at.format("%Y%m%d-%H%M%S").to_string());
    format!("{}.{}", parts.join("-"), format.extension())
}

async fn remove_file_if_exists(path: &Path) {
    if let Err(error) = tokio::fs::remove_file(path).await {
        if error.kind() != std::io::ErrorKind::NotFound {
            tracing::warn!("Failed to remove export file {}: {}", path.display(), error);
        }
    }
}
