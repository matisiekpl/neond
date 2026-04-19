use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use neon_pageserver_client::mgmt_api::ForceAwaitLogicalSize;
use neon_pageserver_api::shard::TenantShardId;
use neon_utils::id::{TenantId, TimelineId};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::mgmt::compute::ComputeEndpointStatus;
use crate::mgmt::dto::config::Config;
use crate::mgmt::dto::daemon_response::{
    DaemonResponse, LocalStorageInfo, MappingInfo, PendingShutdownInfo, RemoteStorageInfo,
    StorageInfo,
};
use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::repository::branch::BranchRepository;
use crate::mgmt::repository::organization::OrganizationRepository;
use crate::mgmt::repository::project::ProjectRepository;
use crate::mgmt::service::branch::BranchService;
use crate::mgmt::service::endpoint::EndpointService;

#[derive(Clone)]
struct PendingShutdown {
    wait_for_checkpoints: bool,
    requested_at: chrono::DateTime<Utc>,
}

pub struct DaemonService {
    config: Config,
    pageserver_client: Arc<neon_pageserver_client::mgmt_api::Client>,
    endpoint_service: Arc<EndpointService>,
    branch_service: Arc<BranchService>,
    branch_repo: Arc<BranchRepository>,
    project_repo: Arc<ProjectRepository>,
    org_repo: Arc<OrganizationRepository>,
    pending_shutdown: Arc<Mutex<Option<PendingShutdown>>>,
    shutdown_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    shutdown_token: CancellationToken,
}

impl DaemonService {
    pub fn new(
        config: Config,
        pageserver_client: Arc<neon_pageserver_client::mgmt_api::Client>,
        endpoint_service: Arc<EndpointService>,
        branch_service: Arc<BranchService>,
        branch_repo: Arc<BranchRepository>,
        project_repo: Arc<ProjectRepository>,
        org_repo: Arc<OrganizationRepository>,
        shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            config,
            pageserver_client,
            endpoint_service,
            branch_service,
            branch_repo,
            project_repo,
            org_repo,
            pending_shutdown: Arc::new(Mutex::new(None)),
            shutdown_task: Arc::new(Mutex::new(None)),
            shutdown_token,
        }
    }

    pub fn shutdown_token(&self) -> CancellationToken {
        self.shutdown_token.clone()
    }

    pub async fn request_shutdown(self: Arc<Self>, wait_for_checkpoints: bool) -> Result<()> {
        let mut pending = self.pending_shutdown.lock().await;
        if pending.is_some() {
            return Err(AppError::Conflict("Shutdown already pending".into()));
        }
        *pending = Some(PendingShutdown {
            wait_for_checkpoints,
            requested_at: Utc::now(),
        });
        drop(pending);

        let service = Arc::clone(&self);
        let handle = tokio::spawn(async move {
            if wait_for_checkpoints {
                loop {
                    match service.branch_service.check_branches_durability().await {
                        Ok(status) if status.all_in_sync => break,
                        _ => {
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                }
            }
            service.endpoint_service.shutdown_all().await;
            service.shutdown_token.cancel();
        });

        let mut task = self.shutdown_task.lock().await;
        *task = Some(handle);

        Ok(())
    }

    pub async fn cancel_shutdown(&self) -> Result<()> {
        let mut task = self.shutdown_task.lock().await;
        match task.take() {
            None => Err(AppError::Conflict("No pending shutdown".into())),
            Some(handle) => {
                handle.abort();
                let mut pending = self.pending_shutdown.lock().await;
                *pending = None;
                Ok(())
            }
        }
    }

    pub async fn get_state(&self) -> Result<DaemonResponse> {
        let storage = match &self.config.remote_storage_config {
            Some(remote) => {
                let aws_access_key_id = std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_default();
                StorageInfo::Remote(RemoteStorageInfo {
                    bucket: remote.bucket.clone(),
                    region: remote.region.clone(),
                    aws_access_key_id,
                })
            }
            None => {
                let stat = fs2::statvfs(&self.config.daemon_directory)
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                let total = stat.total_space();
                let free = stat.available_space();
                let used = total.saturating_sub(free);
                let used_percent = if total > 0 {
                    used as f64 / total as f64 * 100.0
                } else {
                    0.0
                };
                StorageInfo::Local(LocalStorageInfo {
                    used_bytes: used,
                    free_bytes: free,
                    used_percent,
                })
            }
        };

        let organizations = self.org_repo.list_all().await?;
        let mut mappings = Vec::new();

        for organization in organizations {
            let projects = self.project_repo.list_by_org_id(organization.id).await?;
            for project in projects {
                let tenant_id =
                    TenantId::from_str(project.id.as_simple().to_string().as_str())
                        .map_err(|_| AppError::TenantIdInvalid {
                            value: project.id.to_string(),
                        })?;

                let tenant_shard_id = TenantShardId::unsharded(tenant_id);
                let token = self
                    .config
                    .component_auth
                    .generate_token(neon_utils::auth::Scope::PageServerApi, None)?;
                let pageserver_http_client = reqwest::Client::new();
                let config_resp = pageserver_http_client
                    .get(format!(
                        "http://127.0.0.1:1234/v1/tenant/{tenant_shard_id}/config"
                    ))
                    .header("Authorization", format!("Bearer {}", token))
                    .send()
                    .await
                    .ok();
                let checkpoint_timeout = if let Some(resp) = config_resp {
                    let val: serde_json::Value = resp.json().await.unwrap_or_default();
                    val.get("tenant_specific_overrides")
                        .and_then(|overrides| overrides.get("checkpoint_timeout"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| humantime::parse_duration(s).ok())
                } else {
                    None
                };

                let branches = self.branch_repo.list_by_project_id(project.id).await?;
                for branch in branches {
                    let timeline_id =
                        TimelineId::from_str(branch.timeline_id.as_simple().to_string().as_str())
                            .map_err(|_| AppError::TimelineIdInvalid {
                                value: branch.timeline_id.to_string(),
                            })?;

                    let timeline_info = self
                        .pageserver_client
                        .timeline_info(
                            TenantShardId::unsharded(tenant_id),
                            timeline_id,
                            ForceAwaitLogicalSize::Yes,
                        )
                        .await
                        .map_err(|error| AppError::PageserverApiFailed {
                            operation: "timeline_info".to_string(),
                            reason: error.to_string(),
                        })?;

                    let endpoint_info =
                        self.endpoint_service.get_endpoint_info(branch.id).await;
                    let endpoint_status = endpoint_info
                        .as_ref()
                        .map(|info| info.status.clone())
                        .unwrap_or(ComputeEndpointStatus::Stopped);
                    let port = endpoint_info.as_ref().map(|info| info.port);
                    let sni = self
                        .config
                        .hostname
                        .as_ref()
                        .map(|host| format!("{}.{}", branch.slug, host));

                    mappings.push(MappingInfo {
                        branch_id: branch.id,
                        organization_id: organization.id,
                        organization_name: organization.name.clone(),
                        project_id: project.id,
                        project_name: project.name.clone(),
                        branch_name: branch.name.clone(),
                        slug: branch.slug.clone(),
                        endpoint_status,
                        port,
                        sni,
                        last_record_lsn: timeline_info.last_record_lsn,
                        remote_consistent_lsn_visible: timeline_info.remote_consistent_lsn_visible,
                        current_logical_size: timeline_info.current_logical_size,
                        checkpoint_timeout,
                    });
                }
            }
        }

        let pending_shutdown = {
            let pending = self.pending_shutdown.lock().await;
            pending.as_ref().map(|p| PendingShutdownInfo {
                wait_for_checkpoints: p.wait_for_checkpoints,
                requested_at: p.requested_at,
            })
        };

        let max_checkpoint_timeout = self
            .branch_service
            .check_branches_durability()
            .await
            .ok()
            .and_then(|s| s.max_checkpoint_timeout);

        Ok(DaemonResponse {
            hostname: self.config.hostname.clone(),
            build_version: env!("GIT_COMMIT_HASH").to_string(),
            storage,
            mappings,
            pending_shutdown,
            max_checkpoint_timeout,
        })
    }
}