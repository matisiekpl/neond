use std::sync::Arc;
use uuid::Uuid;

use crate::mgmt::dto::branch_response::BranchResponse;
use crate::mgmt::dto::create_branch_request::CreateBranchRequest;
use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::dto::update_branch_request::UpdateBranchRequest;
use crate::mgmt::repository::branch::BranchRepository;
use crate::mgmt::repository::project::ProjectRepository;
use crate::mgmt::service::membership::MembershipService;

pub struct BranchService {
    branch_repo: Arc<BranchRepository>,
    project_repo: Arc<ProjectRepository>,
    membership_service: Arc<MembershipService>,
}

impl BranchService {
    pub fn new(
        branch_repo: Arc<BranchRepository>,
        project_repo: Arc<ProjectRepository>,
        membership_service: Arc<MembershipService>,
    ) -> Self {
        Self {
            branch_repo,
            project_repo,
            membership_service,
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

        if let Some(parent_id) = req.parent_branch_id {
            let parent = self
                .branch_repo
                .find_by_id(parent_id)
                .await?
                .ok_or(AppError::NotFound)?;

            if parent.project_id != project_id {
                return Err(AppError::NotFound);
            }
        }

        let id = Uuid::new_v4();
        let timeline_id = Uuid::new_v4();
        let branch = self
            .branch_repo
            .create(id, project_id, &req.name, req.parent_branch_id, timeline_id)
            .await?;

        Ok(BranchResponse {
            id: branch.id,
            project_id: branch.project_id,
            name: branch.name,
            parent_branch_id: branch.parent_branch_id,
            timeline_id: branch.timeline_id,
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

        Ok(BranchResponse {
            id: branch.id,
            project_id: branch.project_id,
            name: branch.name,
            parent_branch_id: branch.parent_branch_id,
            timeline_id: branch.timeline_id,
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

        Ok(branches
            .into_iter()
            .map(|b| BranchResponse {
                id: b.id,
                project_id: b.project_id,
                name: b.name,
                parent_branch_id: b.parent_branch_id,
                timeline_id: b.timeline_id,
            })
            .collect())
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

        Ok(BranchResponse {
            id: updated.id,
            project_id: updated.project_id,
            name: updated.name,
            parent_branch_id: updated.parent_branch_id,
            timeline_id: updated.timeline_id,
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

        self.branch_repo.delete(branch_id).await
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
