use neon_pageserver_api::controller_api::TenantCreateRequest;
use neon_utils::id::TenantId;
use neon_utils::shard::TenantShardId;
use reqwest::Method;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use crate::mgmt::dto::config::Config;
use crate::mgmt::dto::create_project_request::CreateProjectRequest;
use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::dto::project_response::ProjectResponse;
use crate::mgmt::dto::update_project_request::UpdateProjectRequest;
use crate::mgmt::model::project::PgVersion;
use crate::mgmt::repository::organization::OrganizationRepository;
use crate::mgmt::repository::project::ProjectRepository;
use crate::mgmt::service::membership::MembershipService;

pub struct ProjectService {
    project_repo: Arc<ProjectRepository>,
    org_repo: Arc<OrganizationRepository>,
    membership_service: Arc<MembershipService>,
    pageserver_client: Arc<neon_pageserver_client::mgmt_api::Client>,
    config: Config,
}

impl ProjectService {
    pub fn new(
        project_repo: Arc<ProjectRepository>,
        org_repo: Arc<OrganizationRepository>,
        membership_service: Arc<MembershipService>,
        pageserver_client: Arc<neon_pageserver_client::mgmt_api::Client>,
        config: Config,
    ) -> Self {
        Self {
            project_repo,
            org_repo,
            membership_service,
            pageserver_client,
            config,
        }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        req: CreateProjectRequest,
    ) -> Result<ProjectResponse> {
        Self::validate_project_name(&req.name)?;

        let _ = self
            .org_repo
            .find_by_id(org_id)
            .await?
            .ok_or(AppError::NotFound)?;

        self.membership_service
            .verify_membership(user_id, org_id)
            .await?;

        let tenant_id = TenantId::generate();
        let project_id = Uuid::from_str(tenant_id.to_string().as_str())
            .map_err(|_| AppError::Internal("Invalid project id".to_string()))?;
        let pg_version = req.pg_version.unwrap_or(PgVersion::V17);
        let project = self
            .project_repo
            .create(project_id, org_id, &req.name, pg_version)
            .await?;

        let tenant_create_request = TenantCreateRequest {
            new_tenant_id: TenantShardId::unsharded(tenant_id),
            generation: None,
            shard_parameters: Default::default(),
            placement_policy: None,
            config: Default::default(),
        };

        let token = self
            .config
            .component_auth
            .generate_token(neon_utils::auth::Scope::PageServerApi, None);
        let pageserver_http_client = reqwest::Client::new();
        let response = pageserver_http_client
            .request(Method::POST, "http://127.0.0.1:1234/v1/tenant")
            .header("Authorization", format!("Bearer {}", token))
            .json(&tenant_create_request)
            .send()
            .await
            .unwrap();

        if response.status().as_u16() != 201 {
            return Err(AppError::Internal("Failed to create tenant".into()));
        }

        Ok(ProjectResponse {
            id: project.id,
            organization_id: project.organization_id,
            name: project.name,
            pg_version: project.pg_version,
        })
    }

    pub async fn get(&self, user_id: Uuid, org_id: Uuid, id: Uuid) -> Result<ProjectResponse> {
        self.membership_service
            .verify_membership(user_id, org_id)
            .await?;

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
            pg_version: project.pg_version,
        })
    }

    pub async fn list(&self, user_id: Uuid, org_id: Uuid) -> Result<Vec<ProjectResponse>> {
        let _ = self
            .org_repo
            .find_by_id(org_id)
            .await?
            .ok_or(AppError::NotFound)?;

        self.membership_service
            .verify_membership(user_id, org_id)
            .await?;

        let projects = self.project_repo.list_by_org_id(org_id).await?;

        Ok(projects
            .into_iter()
            .map(|p| ProjectResponse {
                id: p.id,
                organization_id: p.organization_id,
                name: p.name,
                pg_version: p.pg_version,
            })
            .collect())
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        org_id: Uuid,
        id: Uuid,
        req: UpdateProjectRequest,
    ) -> Result<ProjectResponse> {
        Self::validate_project_name(&req.name)?;

        self.membership_service
            .verify_membership(user_id, org_id)
            .await?;

        let project = self
            .project_repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)?;

        if project.organization_id != org_id {
            return Err(AppError::NotFound);
        }

        let updated = self.project_repo.update(id, &req.name).await?;

        Ok(ProjectResponse {
            id: updated.id,
            organization_id: updated.organization_id,
            name: updated.name,
            pg_version: updated.pg_version,
        })
    }

    pub async fn delete(&self, user_id: Uuid, org_id: Uuid, id: Uuid) -> Result<()> {
        self.membership_service
            .verify_membership(user_id, org_id)
            .await?;

        let project = self
            .project_repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::NotFound)?;

        if project.organization_id != org_id {
            return Err(AppError::NotFound);
        }

        let tenant_id = TenantId::from_str(project.id.as_simple().to_string().as_str())
            .map_err(|_| AppError::Internal("Invalid tenant id".to_string()))?;

        let mut status_code = 0;
        while status_code != 200 {
            status_code = self
                .pageserver_client
                .tenant_delete(TenantShardId::unsharded(tenant_id))
                .await
                .map_err(|_| AppError::Internal("Failed to delete tenant".to_string()))?
                .as_u16();
            if status_code != 500 && status_code != 503 && status_code != 409 {
                break;
            }
        }

        self.project_repo.delete(id).await
    }

    fn validate_project_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(AppError::Internal("Project name cannot be empty".into()));
        }

        if name.len() > 255 {
            return Err(AppError::Internal(
                "Project name is too long (max 255 characters)".into(),
            ));
        }

        Ok(())
    }
}
