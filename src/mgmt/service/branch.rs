use neon_pageserver_api::models::{TimelineCreateRequest, TimelineCreateRequestMode};
use neon_pageserver_client::mgmt_api::ForceAwaitLogicalSize;
use neon_utils::id::{TenantId, TimelineId};
use neon_utils::shard::TenantShardId;
use rand::Rng;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use crate::mgmt::dto::branch_response::BranchResponse;
use crate::mgmt::dto::create_branch_request::CreateBranchRequest;
use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::dto::update_branch_request::UpdateBranchRequest;
use crate::mgmt::repository::branch::BranchRepository;
use crate::mgmt::repository::project::ProjectRepository;
use crate::mgmt::service::endpoint::EndpointService;
use crate::mgmt::service::membership::MembershipService;

pub struct BranchService {
    branch_repo: Arc<BranchRepository>,
    project_repo: Arc<ProjectRepository>,
    membership_service: Arc<MembershipService>,
    pageserver_client: Arc<neon_pageserver_client::mgmt_api::Client>,
    endpoint_service: Arc<EndpointService>,
}

impl BranchService {
    pub fn new(
        branch_repo: Arc<BranchRepository>,
        project_repo: Arc<ProjectRepository>,
        membership_service: Arc<MembershipService>,
        pageserver_client: Arc<neon_pageserver_client::mgmt_api::Client>,
        endpoint_service: Arc<EndpointService>,
    ) -> Self {
        Self {
            branch_repo,
            project_repo,
            membership_service,
            pageserver_client,
            endpoint_service,
        }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        project_id: Uuid,
        req: CreateBranchRequest,
    ) -> Result<BranchResponse> {
        Self::validate_branch_name(&req.name)?;

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

        let mode = if let Some(parent_id) = req.parent_branch_id {
            let parent = self
                .branch_repo
                .find_by_id(parent_id)
                .await?
                .ok_or(AppError::NotFound)?;

            if parent.project_id != project_id {
                return Err(AppError::NotFound);
            }

            let ancestor_timeline_id =
                TimelineId::from_str(parent.timeline_id.as_simple().to_string().as_str())
                    .map_err(|_| AppError::Internal("Invalid parent timeline id".into()))?;

            TimelineCreateRequestMode::Branch {
                ancestor_timeline_id,
                ancestor_start_lsn: None,
                pg_version: None,
                read_only: false,
            }
        } else {
            TimelineCreateRequestMode::Bootstrap {
                existing_initdb_timeline_id: None,
                pg_version: None,
            }
        };

        let new_timeline_id = TimelineId::generate();
        let timeline_uuid = Uuid::from_str(new_timeline_id.to_string().as_str())
            .map_err(|_| AppError::Internal("Invalid timeline id".into()))?;

        let tenant_id = TenantId::from_str(project_id.as_simple().to_string().as_str())
            .map_err(|_| AppError::Internal("Invalid tenant id".into()))?;

        self.pageserver_client
            .timeline_create(
                TenantShardId::unsharded(tenant_id),
                &TimelineCreateRequest {
                    new_timeline_id,
                    mode,
                },
            )
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create timeline: {e}")))?;

        let id = Uuid::new_v4();
        let password = Self::generate_password();

        let branch = self
            .branch_repo
            .create(
                id,
                project_id,
                &req.name,
                req.parent_branch_id,
                timeline_uuid,
                &password,
            )
            .await?;

        let endpoint_status = self.endpoint_service.get_status_for_branch(branch.id).await;

        Ok(BranchResponse {
            id: branch.id,
            project_id: branch.project_id,
            name: branch.name,
            parent_branch_id: branch.parent_branch_id,
            timeline_id: branch.timeline_id,
            endpoint_status,
            remote_consistent_lsn_visible: Default::default(),
            last_record_lsn: Default::default(),
        })
    }

    pub async fn get(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
    ) -> Result<BranchResponse> {
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

        let endpoint_status = self.endpoint_service.get_status_for_branch(branch.id).await;

        Ok(BranchResponse {
            id: branch.id,
            project_id: branch.project_id,
            name: branch.name,
            parent_branch_id: branch.parent_branch_id,
            timeline_id: branch.timeline_id,
            endpoint_status,
            remote_consistent_lsn_visible: Default::default(),
            last_record_lsn: Default::default(),
        })
    }

    pub async fn list(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        project_id: Uuid,
    ) -> Result<Vec<BranchResponse>> {
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

        let branches = self.branch_repo.list_by_project_id(project_id).await?;

        let mut results = Vec::with_capacity(branches.len());

        let tenant_id = TenantId::from_str(project_id.as_simple().to_string().as_str())
            .map_err(|_| AppError::Internal("Invalid tenant id".into()))?;

        for b in branches {
            let timeline_id = TimelineId::from_str(b.timeline_id.as_simple().to_string().as_str())
                .map_err(|_| AppError::Internal("Invalid timeline id".into()))?;

            let timeline_info = self
                .pageserver_client
                .timeline_info(
                    TenantShardId::unsharded(tenant_id),
                    timeline_id,
                    ForceAwaitLogicalSize::No,
                )
                .await
                .unwrap();

            let endpoint_status = self.endpoint_service.get_status_for_branch(b.id).await;
            results.push(BranchResponse {
                id: b.id,
                project_id: b.project_id,
                name: b.name,
                parent_branch_id: b.parent_branch_id,
                timeline_id: b.timeline_id,
                endpoint_status,
                remote_consistent_lsn_visible: timeline_info.remote_consistent_lsn_visible,
                last_record_lsn: timeline_info.last_record_lsn,
            });
        }

        Ok(results)
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
        req: UpdateBranchRequest,
    ) -> Result<BranchResponse> {
        Self::validate_branch_name(&req.name)?;

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

        let updated = self.branch_repo.update(branch_id, &req.name).await?;

        let endpoint_status = self
            .endpoint_service
            .get_status_for_branch(updated.id)
            .await;

        Ok(BranchResponse {
            id: updated.id,
            project_id: updated.project_id,
            name: updated.name,
            parent_branch_id: updated.parent_branch_id,
            timeline_id: updated.timeline_id,
            endpoint_status,
            remote_consistent_lsn_visible: Default::default(),
            last_record_lsn: Default::default(),
        })
    }

    pub async fn delete(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
    ) -> Result<()> {
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

        let tenant_id = TenantId::from_str(project_id.as_simple().to_string().as_str())
            .map_err(|_| AppError::Internal("Invalid tenant id".into()))?;

        let timeline_id = TimelineId::from_str(branch.timeline_id.as_simple().to_string().as_str())
            .map_err(|_| AppError::Internal("Invalid timeline id".into()))?;

        let mut status_code = 0;
        loop {
            status_code = self
                .pageserver_client
                .timeline_delete(TenantShardId::unsharded(tenant_id), timeline_id)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to delete timeline: {e}")))?
                .as_u16();
            if status_code != 500 && status_code != 503 && status_code != 409 {
                break;
            }
        }

        if status_code != 200 && status_code != 404 {
            return Err(AppError::Internal(format!(
                "Unexpected status code from pageserver: {status_code}"
            )));
        }

        self.branch_repo.delete(branch_id).await
    }

    fn generate_password() -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        const PASSWORD_LEN: usize = 32;
        let mut rng = rand::rng();
        (0..PASSWORD_LEN)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    fn validate_branch_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(AppError::Internal("Branch name cannot be empty".into()));
        }

        if name.len() > 255 {
            return Err(AppError::Internal(
                "Branch name is too long (max 255 characters)".into(),
            ));
        }

        Ok(())
    }
}
