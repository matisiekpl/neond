use std::str::FromStr;
use std::sync::Arc;

use neon_pageserver_client::mgmt_api::ForceAwaitLogicalSize;
use neon_pageserver_api::shard::TenantShardId;
use neon_utils::id::{TenantId, TimelineId};

use crate::mgmt::compute::ComputeEndpointStatus;
use crate::mgmt::dto::config::Config;
use crate::mgmt::dto::daemon_response::{
    DaemonResponse, LocalStorageInfo, MappingInfo, RemoteStorageInfo, StorageInfo,
};
use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::repository::branch::BranchRepository;
use crate::mgmt::repository::organization::OrganizationRepository;
use crate::mgmt::repository::project::ProjectRepository;
use crate::mgmt::service::endpoint::EndpointService;

pub struct DaemonService {
    config: Config,
    pageserver_client: Arc<neon_pageserver_client::mgmt_api::Client>,
    endpoint_service: Arc<EndpointService>,
    branch_repo: Arc<BranchRepository>,
    project_repo: Arc<ProjectRepository>,
    org_repo: Arc<OrganizationRepository>,
}

impl DaemonService {
    pub fn new(
        config: Config,
        pageserver_client: Arc<neon_pageserver_client::mgmt_api::Client>,
        endpoint_service: Arc<EndpointService>,
        branch_repo: Arc<BranchRepository>,
        project_repo: Arc<ProjectRepository>,
        org_repo: Arc<OrganizationRepository>,
    ) -> Self {
        Self {
            config,
            pageserver_client,
            endpoint_service,
            branch_repo,
            project_repo,
            org_repo,
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
                        .map_err(|_| AppError::Internal("Invalid tenant id".into()))?;

                let branches = self.branch_repo.list_by_project_id(project.id).await?;
                for branch in branches {
                    let timeline_id =
                        TimelineId::from_str(branch.timeline_id.as_simple().to_string().as_str())
                            .map_err(|_| AppError::Internal("Invalid timeline id".into()))?;

                    let timeline_info = self
                        .pageserver_client
                        .timeline_info(
                            TenantShardId::unsharded(tenant_id),
                            timeline_id,
                            ForceAwaitLogicalSize::Yes,
                        )
                        .await
                        .unwrap();

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
                    });
                }
            }
        }

        Ok(DaemonResponse {
            hostname: self.config.hostname.clone(),
            build_version: env!("GIT_COMMIT_HASH").to_string(),
            storage,
            mappings,
        })
    }
}