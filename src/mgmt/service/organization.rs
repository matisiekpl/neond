use std::sync::Arc;
use uuid::Uuid;

use crate::mgmt::dto::create_organization_request::CreateOrganizationRequest;
use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::dto::organization_response::OrganizationResponse;
use crate::mgmt::dto::update_organization_request::UpdateOrganizationRequest;
use crate::mgmt::dto::user_response::UserResponse;
use crate::mgmt::repository::membership::MembershipRepository;
use crate::mgmt::repository::organization::OrganizationRepository;
use crate::mgmt::repository::project::ProjectRepository;
use crate::mgmt::repository::user::UserRepository;
use crate::mgmt::service::membership::MembershipService;
use crate::mgmt::service::project::ProjectService;

pub struct OrganizationService {
    org_repo: Arc<OrganizationRepository>,
    project_repo: Arc<ProjectRepository>,
    membership_repo: Arc<MembershipRepository>,
    membership_service: Arc<MembershipService>,
    user_repo: Arc<UserRepository>,
    project_service: Arc<ProjectService>,
}

impl OrganizationService {
    pub fn new(
        org_repo: Arc<OrganizationRepository>,
        project_repo: Arc<ProjectRepository>,
        membership_repo: Arc<MembershipRepository>,
        membership_service: Arc<MembershipService>,
        user_repo: Arc<UserRepository>,
        project_service: Arc<ProjectService>,
    ) -> Self {
        Self {
            org_repo,
            project_repo,
            membership_repo,
            membership_service,
            user_repo,
            project_service,
        }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        req: CreateOrganizationRequest,
    ) -> Result<OrganizationResponse> {
        Self::validate_organization_name(&req.name)?;

        let org_id = Uuid::new_v4();
        let org = self.org_repo.create(org_id, &req.name).await?;

        self.membership_repo.create(user_id, org_id).await?;

        Ok(OrganizationResponse {
            id: org.id,
            name: org.name,
            created_at: org.created_at,
            updated_at: org.updated_at,
        })
    }

    pub async fn get(&self, user_id: Uuid, id: Uuid) -> Result<OrganizationResponse> {
        self.membership_service
            .verify_membership(user_id, id)
            .await?;

        let org = self
            .org_repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)?;
        Ok(OrganizationResponse {
            id: org.id,
            name: org.name,
            created_at: org.created_at,
            updated_at: org.updated_at,
        })
    }

    pub async fn list(&self, user_id: Uuid) -> Result<Vec<OrganizationResponse>> {
        let memberships = self.membership_repo.list_by_user_id(user_id).await?;

        let mut organizations = Vec::new();

        for membership in memberships {
            if let Ok(Some(org)) = self.org_repo.find_by_id(membership.organization_id).await {
                organizations.push(OrganizationResponse {
                    id: org.id,
                    name: org.name,
                    created_at: org.created_at,
                    updated_at: org.updated_at,
                });
            }
        }

        Ok(organizations)
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        id: Uuid,
        req: UpdateOrganizationRequest,
    ) -> Result<OrganizationResponse> {
        let _ = self
            .org_repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)?;

        self.membership_service
            .verify_membership(user_id, id)
            .await?;

        Self::validate_organization_name(&req.name)?;

        let org = self.org_repo.update(id, &req.name).await?;
        Ok(OrganizationResponse {
            id: org.id,
            name: org.name,
            created_at: org.created_at,
            updated_at: org.updated_at,
        })
    }

    pub async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<()> {
        let _ = self
            .org_repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)?;

        self.membership_service
            .verify_membership(user_id, id)
            .await?;

        let projects = self.project_repo.list_by_org_id(id).await?;
        for project in projects {
            self.project_service.delete_project(project).await?;
        }

        self.membership_repo.delete_by_org_id(id).await?;
        self.org_repo.delete(id).await
    }

    pub async fn assign_member(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<()> {
        let _ = self
            .org_repo
            .find_by_id(org_id)
            .await?
            .ok_or(AppError::NotFound)?;

        self.membership_service
            .verify_membership(user_id, org_id)
            .await?;

        let already_member = self.membership_repo.exists(target_user_id, org_id).await?;

        if already_member {
            return Err(AppError::Conflict(
                "User is already a member of this organization".into(),
            ));
        }

        self.membership_repo.create(target_user_id, org_id).await?;

        Ok(())
    }

    pub async fn revoke_member(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<()> {
        let _ = self
            .org_repo
            .find_by_id(org_id)
            .await?
            .ok_or(AppError::NotFound)?;

        self.membership_service
            .verify_membership(user_id, org_id)
            .await?;

        let is_member = self.membership_repo.exists(target_user_id, org_id).await?;

        if !is_member {
            return Err(AppError::NotFound);
        }

        let members = self.membership_repo.list_by_org_id(org_id).await?;

        if members.len() <= 1 {
            return Err(AppError::Internal(
                "Cannot revoke last member from organization".into(),
            ));
        }

        self.membership_repo.delete(target_user_id, org_id).await
    }

    pub async fn list_members(&self, user_id: Uuid, org_id: Uuid) -> Result<Vec<UserResponse>> {
        self.membership_service
            .verify_membership(user_id, org_id)
            .await?;

        let memberships = self.membership_repo.list_by_org_id(org_id).await?;

        let mut users = Vec::new();
        for m in memberships {
            let user = self
                .user_repo
                .find_by_id(m.user_id)
                .await?
                .ok_or(AppError::Internal(
                    "Member user record missing".into(),
                ))?;
            users.push(UserResponse {
                id: user.id,
                name: user.name,
                email: user.email,
                created_at: user.created_at,
                updated_at: user.updated_at,
            });
        }
        Ok(users)
    }

    pub async fn assign_member_by_email(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        email: &str,
    ) -> Result<()> {
        let email = email.trim();
        if email.is_empty() {
            return Err(AppError::Internal("Email cannot be empty".into()));
        }
        let target = self
            .user_repo
            .find_by_email(email)
            .await?
            .ok_or(AppError::NotFound)?;
        self.assign_member(user_id, org_id, target.id).await
    }

    fn validate_organization_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(AppError::Internal(
                "Organization name cannot be empty".into(),
            ));
        }

        if name.len() > 255 {
            return Err(AppError::Internal(
                "Organization name is too long (max 255 characters)".into(),
            ));
        }

        Ok(())
    }
}
