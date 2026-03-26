use std::collections::HashMap;
use std::fmt::format;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::mgmt::compute::{ComputeEndpoint, ComputeEndpointStatus};
use crate::mgmt::dto::config::Config;
use crate::mgmt::dto::endpoint_response::EndpointResponse;
use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::repository::branch::BranchRepository;
use crate::mgmt::repository::project::ProjectRepository;
use crate::mgmt::service::membership::MembershipService;
use pg_sni_muxer::PgSniMuxer;
use tokio::net::TcpListener;

pub struct EndpointService {
    endpoints: Arc<Mutex<HashMap<Uuid, ComputeEndpoint>>>,
    branch_repo: Arc<BranchRepository>,
    project_repo: Arc<ProjectRepository>,
    membership_service: Arc<MembershipService>,
    config: Config,
    pg_proxy: Arc<PgSniMuxer>,
}

impl EndpointService {
    pub fn new(
        config: Config,
        branch_repo: Arc<BranchRepository>,
        project_repo: Arc<ProjectRepository>,
        membership_service: Arc<MembershipService>,
    ) -> Self {
        Self {
            config,
            endpoints: Arc::new(Mutex::new(HashMap::new())),
            branch_repo,
            project_repo,
            membership_service,
            pg_proxy: Arc::new(PgSniMuxer::new()),
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
            match existing.get_status() {
                ComputeEndpointStatus::Running => {
                    return Err(AppError::Conflict(
                        "Endpoint for this branch is already running".into(),
                    ));
                }
                ComputeEndpointStatus::Starting => {
                    return Err(AppError::Conflict(
                        "Endpoint for this branch is already starting".into(),
                    ));
                }
                ComputeEndpointStatus::Stopping => {
                    return Err(AppError::Conflict(
                        "Endpoint for this branch is currently stopping".into(),
                    ));
                }
                ComputeEndpointStatus::Stopped | ComputeEndpointStatus::Failed => {}
            }
        }

        let mut endpoint =
            ComputeEndpoint::new(self.config.clone(), branch.clone(), project.pg_version).map_err(
                |e| AppError::Internal(format!("Failed to create compute endpoint: {e}")),
            )?;

        endpoint
            .launch()
            .map_err(|e| AppError::Internal(format!("Failed to launch compute endpoint: {e}")))?;

        let response = EndpointResponse {
            branch_id,
            status: endpoint.get_status(),
            port: endpoint.get_port(),
        };

        if let Some(ref hostname) = self.config.hostname {
            let sni_hostname = format!("{}.{}", branch.slug, hostname);
            let backend_addr: SocketAddr = format!("127.0.0.1:{}", response.port)
                .parse()
                .expect("valid socket addr");
            self.pg_proxy.set_mapping(sni_hostname, backend_addr).await;
        }

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

        if let Some(ref hostname) = self.config.hostname {
            let sni_hostname = format!("{}.{}", branch.slug, hostname);
            self.pg_proxy.remove_mapping(&sni_hostname).await;
        }

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
            let status = endpoint.get_status();
            if status == ComputeEndpointStatus::Running || status == ComputeEndpointStatus::Starting
            {
                if let Err(e) = endpoint.shutdown() {
                    tracing::error!(
                        "Failed to shutdown endpoint for branch {}: {}",
                        branch_id,
                        e
                    );
                } else {
                    tracing::info!("Shutdown endpoint for branch {}", branch_id);
                    if let Some(ref hostname) = self.config.hostname {
                        let sni_hostname = format!("{}.{}", endpoint.get_branch().slug, hostname);
                        self.pg_proxy.remove_mapping(&sni_hostname).await;
                    }
                }
            }
        }
    }

    pub async fn get_status_for_branch(&self, branch_id: Uuid) -> ComputeEndpointStatus {
        let endpoints = self.endpoints.lock().await;
        endpoints
            .get(&branch_id)
            .map(|e| e.get_status())
            .unwrap_or(ComputeEndpointStatus::Stopped)
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

    pub async fn listen(&self) -> std::result::Result<(), anyhow::Error> {
        if self.config.hostname.is_some() {
            let listener =
                TcpListener::bind(format!("0.0.0.0:{}", self.config.pg_proxy_port)).await?;
            tracing::info!(
                "TLS SNI proxy listening on port 0.0.0.0:{}",
                self.config.pg_proxy_port
            );
            self.pg_proxy.clone().listen(listener).await?;
        }
        Ok(())
    }
}
