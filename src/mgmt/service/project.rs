use std::sync::Arc;
use uuid::Uuid;

use crate::mgmt::dto::create_project_request::CreateProjectRequest;
use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::dto::project_response::ProjectResponse;
use crate::mgmt::dto::update_project_request::UpdateProjectRequest;
use crate::mgmt::repository::organization::OrganizationRepository;
use crate::mgmt::repository::project::ProjectRepository;
use crate::mgmt::service::membership::MembershipService;

pub struct ProjectService {
    project_repo: Arc<ProjectRepository>,
    org_repo: Arc<OrganizationRepository>,
    membership_service: Arc<MembershipService>,
}

impl ProjectService {
    pub fn new(
        project_repo: Arc<ProjectRepository>,
        org_repo: Arc<OrganizationRepository>,
        membership_service: Arc<MembershipService>,
    ) -> Self {
        Self {
            project_repo,
            org_repo,
            membership_service,
        }
    }

    pub async fn create(&self, user_id: Uuid, org_id: Uuid, req: CreateProjectRequest) -> Result<ProjectResponse> {
        Self::validate_project_name(&req.name)?;

        let _ = self
            .org_repo
            .find_by_id(org_id)
            .await?
            .ok_or(AppError::NotFound)?;

        self.membership_service.verify_membership(user_id, org_id).await?;

        let project_id = Uuid::new_v4();
        let project = self
            .project_repo
            .create(project_id, org_id, &req.name)
            .await?;

        Ok(ProjectResponse {
            id: project.id,
            organization_id: project.organization_id,
            name: project.name,
        })
    }

    pub async fn get(&self, user_id: Uuid, org_id: Uuid, id: Uuid) -> Result<ProjectResponse> {
        self.membership_service.verify_membership(user_id, org_id).await?;

        let project = self
            .project_repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)?;

        if project.organization_id != org_id {
            return Err(AppError::NotFound);
        }

        Ok(ProjectResponse {
            id: project.id,
            organization_id: project.organization_id,
            name: project.name,
        })
    }

    pub async fn list(&self, user_id: Uuid, org_id: Uuid) -> Result<Vec<ProjectResponse>> {
        let _ = self
            .org_repo
            .find_by_id(org_id)
            .await?
            .ok_or(AppError::NotFound)?;

        self.membership_service.verify_membership(user_id, org_id).await?;

        let projects = self
            .project_repo
            .list_by_org_id(org_id)
            .await?;

        Ok(projects
            .into_iter()
            .map(|p| ProjectResponse {
                id: p.id,
                organization_id: p.organization_id,
                name: p.name,
            })
            .collect())
    }

    pub async fn update(&self, user_id: Uuid, org_id: Uuid, id: Uuid, req: UpdateProjectRequest) -> Result<ProjectResponse> {
        Self::validate_project_name(&req.name)?;

        self.membership_service.verify_membership(user_id, org_id).await?;

        let project = self
            .project_repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)?;

        if project.organization_id != org_id {
            return Err(AppError::NotFound);
        }

        let updated = self
            .project_repo
            .update(id, &req.name)
            .await?;

        Ok(ProjectResponse {
            id: updated.id,
            organization_id: updated.organization_id,
            name: updated.name,
        })
    }

    pub async fn delete(&self, user_id: Uuid, org_id: Uuid, id: Uuid) -> Result<()> {
        self.membership_service.verify_membership(user_id, org_id).await?;

        let project = self
            .project_repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)?;

        if project.organization_id != org_id {
            return Err(AppError::NotFound);
        }

        self.project_repo.delete(id).await
    }

    fn validate_project_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(AppError::Internal("Project name cannot be empty".into()));
        }

        if name.len() > 255 {
            return Err(AppError::Internal("Project name is too long (max 255 characters)".into()));
        }

        Ok(())
    }
}
