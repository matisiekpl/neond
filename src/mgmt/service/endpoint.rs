use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::mgmt::compute::{ComputeEndpoint, ComputeEndpointStatus};
use crate::mgmt::dto::endpoint_response::EndpointResponse;
use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::repository::branch::BranchRepository;
use crate::mgmt::repository::project::ProjectRepository;
use crate::mgmt::service::membership::MembershipService;

pub struct EndpointService {
    endpoints: Arc<Mutex<HashMap<Uuid, ComputeEndpoint>>>,
    branch_repo: Arc<BranchRepository>,
    project_repo: Arc<ProjectRepository>,
    membership_service: Arc<MembershipService>,
    binaries_directory: PathBuf,
}

impl EndpointService {
    pub fn new(
        branch_repo: Arc<BranchRepository>,
        project_repo: Arc<ProjectRepository>,
        membership_service: Arc<MembershipService>,
        binaries_directory: PathBuf,
    ) -> Self {
        Self {
            endpoints: Arc::new(Mutex::new(HashMap::new())),
            branch_repo,
            project_repo,
            membership_service,
            binaries_directory,
        }
    }

    pub async fn start(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
    ) -> Result<EndpointResponse> {
        self.membership_service
            .verify_membership(user_id, org_id)
            .await?;

        let project = self
            .project_repo
            .find_by_id(project_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if project.organization_id != org_id {
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

        let mut endpoints = self.endpoints.lock().await;

        if let Some(existing) = endpoints.get(&branch_id) {
            if existing.get_status() == ComputeEndpointStatus::Running {
                return Err(AppError::Conflict(
                    "Endpoint for this branch is already running".into(),
                ));
            }
        }

        let mut endpoint = ComputeEndpoint::new(branch, self.binaries_directory.clone())
            .map_err(|e| AppError::Internal(format!("Failed to create compute endpoint: {e}")))?;

        endpoint
            .launch()
            .map_err(|e| AppError::Internal(format!("Failed to launch compute endpoint: {e}")))?;

        let response = EndpointResponse {
            branch_id,
            status: endpoint.get_status(),
            port: endpoint.get_port(),
        };

        endpoints.insert(branch_id, endpoint);

        Ok(response)
    }

    pub async fn stop(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
    ) -> Result<EndpointResponse> {
        self.membership_service
            .verify_membership(user_id, org_id)
            .await?;

        let project = self
            .project_repo
            .find_by_id(project_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if project.organization_id != org_id {
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

        let mut endpoints = self.endpoints.lock().await;

        let endpoint = endpoints.get_mut(&branch_id).ok_or(AppError::NotFound)?;

        endpoint
            .shutdown()
            .map_err(|e| AppError::Internal(format!("Failed to shutdown compute endpoint: {e}")))?;

        let response = EndpointResponse {
            branch_id,
            status: endpoint.get_status(),
            port: endpoint.get_port(),
        };

        Ok(response)
    }

    pub async fn shutdown_all(&self) {
        let mut endpoints = self.endpoints.lock().await;
        for (branch_id, endpoint) in endpoints.iter_mut() {
            if endpoint.get_status() == ComputeEndpointStatus::Running {
                if let Err(e) = endpoint.shutdown() {
                    tracing::error!(
                        "Failed to shutdown endpoint for branch {}: {}",
                        branch_id,
                        e
                    );
                } else {
                    tracing::info!("Shutdown endpoint for branch {}", branch_id);
                }
            }
        }
    }

    pub async fn status(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
    ) -> Result<EndpointResponse> {
        self.membership_service
            .verify_membership(user_id, org_id)
            .await?;

        let project = self
            .project_repo
            .find_by_id(project_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if project.organization_id != org_id {
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

        let endpoints = self.endpoints.lock().await;

        let endpoint = endpoints.get(&branch_id).ok_or(AppError::NotFound)?;

        Ok(EndpointResponse {
            branch_id,
            status: endpoint.get_status(),
            port: endpoint.get_port(),
        })
    }
}
