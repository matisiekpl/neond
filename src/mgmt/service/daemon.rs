use std::sync::Arc;

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
    endpoint_service: Arc<EndpointService>,
    branch_repo: Arc<BranchRepository>,
    project_repo: Arc<ProjectRepository>,
    org_repo: Arc<OrganizationRepository>,
}

impl DaemonService {
    pub fn new(
        config: Config,
        endpoint_service: Arc<EndpointService>,
        branch_repo: Arc<BranchRepository>,
        project_repo: Arc<ProjectRepository>,
        org_repo: Arc<OrganizationRepository>,
    ) -> Self {
        Self {
            config,
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

        let active = self.endpoint_service.get_all_active().await;
        let mut mappings = Vec::with_capacity(active.len());
        for (branch_id, branch_slug, port) in active {
            let branch = self
                .branch_repo
                .find_by_id(branch_id)
                .await?
                .ok_or(AppError::NotFound)?;
            let project = self
                .project_repo
                .find_by_id(branch.project_id)
                .await?
                .ok_or(AppError::NotFound)?;
            let org = self
                .org_repo
                .find_by_id(project.organization_id)
                .await?
                .ok_or(AppError::NotFound)?;
            let sni = self
                .config
                .hostname
                .as_ref()
                .map(|h| format!("{}.{}", branch_slug, h));
            mappings.push(MappingInfo {
                organization_name: org.name,
                project_name: project.name,
                branch_name: branch.name,
                port,
                sni,
            });
        }

        Ok(DaemonResponse { storage, mappings })
    }
}