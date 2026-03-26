use std::collections::HashMap;
use std::fmt::format;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::mgmt::compute::{ComputeEndpoint, ComputeEndpointInfo, ComputeEndpointStatus};
use crate::mgmt::dto::config::Config;
use crate::mgmt::dto::endpoint_response::EndpointResponse;
use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::model::branch::Branch;
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

        let sni_hostname = self
            .config
            .hostname
            .as_ref()
            .map(|hostname| format!("{}.{}", branch.slug, hostname));

        if let Some(ref sni_hostname) = sni_hostname {
            let port = endpoint.get_port();
            let backend_addr: SocketAddr = format!("127.0.0.1:{}", port)
                .parse()
                .expect("valid socket addr");
            self.pg_proxy
                .set_mapping(sni_hostname.clone(), backend_addr)
                .await;
        }

        let launched_status = endpoint.get_status();
        self.branch_repo
            .update_recent_status(branch_id, launched_status)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(
                    "Failed to save recent_status for branch {}: {}",
                    branch_id,
                    e
                );
                branch.clone()
            });

        let response = EndpointResponse {
            branch_id,
            status: launched_status,
            port: endpoint.get_port(),
            sni_hostname,
            password: branch.password.clone(),
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

        let sni_hostname = self
            .config
            .hostname
            .as_ref()
            .map(|hostname| format!("{}.{}", branch.slug, hostname));

        if let Some(ref sni_hostname) = sni_hostname {
            self.pg_proxy.remove_mapping(sni_hostname).await;
        }

        let final_status = endpoint.get_status();
        self.branch_repo
            .update_recent_status(branch_id, final_status)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(
                    "Failed to save recent_status for branch {}: {}",
                    branch_id,
                    e
                );
                branch.clone()
            });

        let response = EndpointResponse {
            branch_id,
            status: final_status,
            port: endpoint.get_port(),
            sni_hostname,
            password: branch.password.clone(),
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

    pub async fn get_endpoint_info(
        &self,
        branch_id: Uuid,
    ) -> Option<ComputeEndpointInfo> {
        let endpoints = self.endpoints.lock().await;
        endpoints
            .get(&branch_id)
            .map(|e| ComputeEndpointInfo {
                status: e.get_status(),
                port: e.get_port(),
            })
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

        let sni_hostname = self
            .config
            .hostname
            .as_ref()
            .map(|hostname| format!("{}.{}", branch.slug, hostname));

        let endpoints = self.endpoints.lock().await;

        let (status, port) = if let Some(endpoint) = endpoints.get(&branch_id) {
            (endpoint.get_status(), endpoint.get_port())
        } else {
            let recent = branch
                .recent_status
                .unwrap_or(ComputeEndpointStatus::Stopped);
            (recent, 0)
        };

        Ok(EndpointResponse {
            branch_id,
            status,
            port,
            sni_hostname,
            password: branch.password.clone(),
        })
    }

    pub async fn recover_running(&self) {
        let branches = match self
            .branch_repo
            .list_all_with_recent_status(ComputeEndpointStatus::Running)
            .await
        {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("Failed to fetch branches for recovery: {}", e);
                return;
            }
        };

        if branches.is_empty() {
            tracing::info!("No endpoints to recover");
            return;
        }

        tracing::info!("Recovering {} running endpoint(s)", branches.len());

        let mut endpoints = self.endpoints.lock().await;

        for branch in branches {
            let project = match self.project_repo.find_by_id(branch.project_id).await {
                Ok(Some(p)) => p,
                Ok(None) => {
                    tracing::warn!(
                        "Project not found for branch {} during recovery, skipping",
                        branch.id
                    );
                    continue;
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to fetch project for branch {} during recovery: {}",
                        branch.id,
                        e
                    );
                    continue;
                }
            };

            let mut endpoint =
                match ComputeEndpoint::new(self.config.clone(), branch.clone(), project.pg_version)
                {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::error!(
                            "Failed to create endpoint for branch {} during recovery: {}",
                            branch.id,
                            e
                        );
                        continue;
                    }
                };

            match endpoint.launch() {
                Ok(()) => {
                    let sni_hostname = self
                        .config
                        .hostname
                        .as_ref()
                        .map(|hostname| format!("{}.{}", branch.slug, hostname));

                    if let Some(ref sni_hostname) = sni_hostname {
                        let port = endpoint.get_port();
                        let backend_addr: std::net::SocketAddr = format!("127.0.0.1:{}", port)
                            .parse()
                            .expect("valid socket addr");
                        self.pg_proxy
                            .set_mapping(sni_hostname.clone(), backend_addr)
                            .await;
                    }

                    tracing::info!("Recovered endpoint for branch {}", branch.id);
                    endpoints.insert(branch.id, endpoint);
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to launch endpoint for branch {} during recovery: {}",
                        branch.id,
                        e
                    );
                    self.branch_repo
                        .update_recent_status(branch.id, ComputeEndpointStatus::Failed)
                        .await
                        .unwrap_or_else(|e| {
                            tracing::warn!(
                                "Failed to save recent_status for branch {}: {}",
                                branch.id,
                                e
                            );
                            branch.clone()
                        });
                }
            }
        }
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
