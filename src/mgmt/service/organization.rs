use std::sync::Arc;
use uuid::Uuid;

use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::dto::create_organization_request::CreateOrganizationRequest;
use crate::mgmt::dto::organization_response::OrganizationResponse;
use crate::mgmt::dto::update_organization_request::UpdateOrganizationRequest;
use crate::mgmt::repository::membership::MembershipRepository;
use crate::mgmt::repository::organization::OrganizationRepository;
use crate::mgmt::service::membership::MembershipService;

pub struct OrganizationService {
    org_repo: Arc<OrganizationRepository>,
    membership_repo: Arc<MembershipRepository>,
    membership_service: Arc<MembershipService>,
}

impl OrganizationService {
    pub fn new(
        org_repo: Arc<OrganizationRepository>,
        membership_repo: Arc<MembershipRepository>,
        membership_service: Arc<MembershipService>,
    ) -> Self {
        Self {
            org_repo,
            membership_repo,
            membership_service,
        }
    }

    pub async fn create(&self, user_id: Uuid, req: CreateOrganizationRequest) -> Result<OrganizationResponse> {
        Self::validate_organization_name(&req.name)?;

        let org_id = Uuid::new_v4();
        let org = self
            .org_repo
            .create(org_id, &req.name)
            .await?;

        self.membership_repo
            .create(user_id, org_id)
            .await?;

        Ok(OrganizationResponse {
            id: org.id,
            name: org.name,
        })
    }

    pub async fn get(&self, user_id: Uuid, id: Uuid) -> Result<OrganizationResponse> {
        self.membership_service.verify_membership(user_id, id).await?;

        let org = self
            .org_repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)?;
        Ok(OrganizationResponse {
            id: org.id,
            name: org.name,
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
                });
            }
        }

        Ok(organizations)
    }

    pub async fn update(&self, user_id: Uuid, id: Uuid, req: UpdateOrganizationRequest) -> Result<OrganizationResponse> {
        let _ = self
            .org_repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)?;

        self.membership_service.verify_membership(user_id, id).await?;

        Self::validate_organization_name(&req.name)?;

        let org = self
            .org_repo
            .update(id, &req.name)
            .await?;
        Ok(OrganizationResponse {
            id: org.id,
            name: org.name,
        })
    }

    pub async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<()> {
        let _ = self
            .org_repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)?;

        self.membership_service.verify_membership(user_id, id).await?;

        self.org_repo.delete(id).await
    }

    pub async fn assign_member(&self, user_id: Uuid, org_id: Uuid, target_user_id: Uuid) -> Result<()> {
        let _ = self
            .org_repo
            .find_by_id(org_id)
            .await?
            .ok_or(AppError::NotFound)?;

        self.membership_service.verify_membership(user_id, org_id).await?;

        let already_member = self
            .membership_repo
            .exists(target_user_id, org_id)
            .await?;

        if already_member {
            return Err(AppError::Conflict("User is already a member of this organization".into()));
        }

        self.membership_repo
            .create(target_user_id, org_id)
            .await?;

        Ok(())
    }

    pub async fn revoke_member(&self, user_id: Uuid, org_id: Uuid, target_user_id: Uuid) -> Result<()> {
        let _ = self
            .org_repo
            .find_by_id(org_id)
            .await?
            .ok_or(AppError::NotFound)?;

        self.membership_service.verify_membership(user_id, org_id).await?;

        let is_member = self
            .membership_repo
            .exists(target_user_id, org_id)
            .await?;

        if !is_member {
            return Err(AppError::NotFound);
        }

        let members = self
            .membership_repo
            .list_by_org_id(org_id)
            .await?;

        if members.len() <= 1 {
            return Err(AppError::Internal("Cannot revoke last member from organization".into()));
        }

        self.membership_repo.delete(target_user_id, org_id).await
    }

    pub async fn list_members(&self, user_id: Uuid, org_id: Uuid) -> Result<Vec<Uuid>> {
        self.membership_service.verify_membership(user_id, org_id).await?;

        let memberships = self
            .membership_repo
            .list_by_org_id(org_id)
            .await?;

        Ok(memberships.into_iter().map(|m| m.user_id).collect())
    }

    fn validate_organization_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(AppError::Internal("Organization name cannot be empty".into()));
        }

        if name.len() > 255 {
            return Err(AppError::Internal("Organization name is too long (max 255 characters)".into()));
        }

        Ok(())
    }
}
