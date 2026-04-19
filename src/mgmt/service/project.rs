use neon_pageserver_api::controller_api::TenantCreateRequest;
use neon_pageserver_api::models::{FieldPatch, TenantConfig, TenantConfigPatch, TenantConfigPatchRequest};
use std::time::Duration;
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
use crate::mgmt::model::project::{PgVersion, Project};
use crate::mgmt::repository::organization::OrganizationRepository;
use crate::mgmt::repository::project::ProjectRepository;
use crate::mgmt::service::branch::BranchService;
use crate::mgmt::service::membership::MembershipService;

pub struct ProjectService {
    project_repo: Arc<ProjectRepository>,
    org_repo: Arc<OrganizationRepository>,
    membership_service: Arc<MembershipService>,
    branch_service: Arc<BranchService>,
    pageserver_client: Arc<neon_pageserver_client::mgmt_api::Client>,
    config: Config,
}

impl ProjectService {
    pub fn new(
        project_repo: Arc<ProjectRepository>,
        org_repo: Arc<OrganizationRepository>,
        membership_service: Arc<MembershipService>,
        branch_service: Arc<BranchService>,
        pageserver_client: Arc<neon_pageserver_client::mgmt_api::Client>,
        config: Config,
    ) -> Self {
        Self {
            project_repo,
            org_repo,
            membership_service,
            branch_service,
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
            .map_err(|error| AppError::ProjectCreationFailed {
                reason: format!("Invalid project id: {error}"),
            })?;
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
            config: TenantConfig {
                gc_period: Some(Duration::from_secs(60 * 60)),            // 1h
                gc_horizon: Some(64 * 1024 * 1024),                       // 64 MB
                pitr_interval: Some(Duration::from_secs(7 * 24 * 60 * 60)), // 7 days
                checkpoint_distance: Some(256 * 1024 * 1024),             // 256 MB
                checkpoint_timeout: Some(Duration::from_secs(5 * 60)),    // 5m
                ..Default::default()
            },
        };

        let token = self
            .config
            .component_auth
            .generate_token(neon_utils::auth::Scope::PageServerApi, None)?;
        let pageserver_http_client = reqwest::Client::new();
        let response = pageserver_http_client
            .request(Method::POST, "http://127.0.0.1:1234/v1/tenant")
            .header("Authorization", format!("Bearer {}", token))
            .json(&tenant_create_request)
            .send()
            .await
            .map_err(|error| AppError::ProjectCreationFailed {
                reason: error.to_string(),
            })?;

        if response.status().as_u16() != 201 {
            return Err(AppError::ProjectCreationFailed {
                reason: format!("pageserver returned status {}", response.status()),
            });
        }

        Ok(ProjectResponse {
            id: project.id,
            organization_id: project.organization_id,
            name: project.name,
            pg_version: project.pg_version,
            created_at: project.created_at,
            updated_at: project.updated_at,
            gc_period: None,
            gc_horizon: None,
            pitr_interval: None,
            checkpoint_distance: None,
            checkpoint_timeout: None,
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

        let tenant_id = TenantId::from_str(project.id.as_simple().to_string().as_str())
            .map_err(|_| AppError::TenantIdInvalid {
                value: project.id.to_string(),
            })?;
        let tenant_shard_id = TenantShardId::unsharded(tenant_id);

        let token = self
            .config
            .component_auth
            .generate_token(neon_utils::auth::Scope::PageServerApi, None)?;
        let config_resp = reqwest::Client::new()
            .get(format!(
                "http://127.0.0.1:1234/v1/tenant/{tenant_shard_id}/config"
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .ok();

        let (gc_period, gc_horizon, pitr_interval, checkpoint_distance, checkpoint_timeout) =
            if let Some(resp) = config_resp {
                let val: serde_json::Value = resp.json().await.unwrap_or_default();
                let overrides = val
                    .get("tenant_specific_overrides")
                    .cloned()
                    .unwrap_or_default();
                (
                    overrides
                        .get("gc_period")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    overrides.get("gc_horizon").and_then(|v| v.as_u64()),
                    overrides
                        .get("pitr_interval")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    overrides
                        .get("checkpoint_distance")
                        .and_then(|v| v.as_u64()),
                    overrides
                        .get("checkpoint_timeout")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                )
            } else {
                (None, None, None, None, None)
            };

        Ok(ProjectResponse {
            id: project.id,
            organization_id: project.organization_id,
            name: project.name,
            pg_version: project.pg_version,
            created_at: project.created_at,
            updated_at: project.updated_at,
            gc_period,
            gc_horizon,
            pitr_interval,
            checkpoint_distance,
            checkpoint_timeout,
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
                created_at: p.created_at,
                updated_at: p.updated_at,
                gc_period: None,
                gc_horizon: None,
                pitr_interval: None,
                checkpoint_distance: None,
                checkpoint_timeout: None,
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
        if let Some(ref name) = req.name {
            Self::validate_project_name(name)?;
        }

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

        if let Some(ref name) = req.name {
            self.project_repo.update(id, name).await?;
        }

        let has_config = req.gc_period.is_some()
            || req.gc_horizon.is_some()
            || req.pitr_interval.is_some()
            || req.checkpoint_distance.is_some()
            || req.checkpoint_timeout.is_some();

        if has_config {
            let tenant_id = TenantId::from_str(id.as_simple().to_string().as_str())
                .map_err(|_| AppError::TenantIdInvalid {
                    value: id.to_string(),
                })?;

            let config = TenantConfigPatch {
                gc_period: req.gc_period.map(FieldPatch::Upsert).unwrap_or_default(),
                gc_horizon: req.gc_horizon.map(FieldPatch::Upsert).unwrap_or_default(),
                pitr_interval: req.pitr_interval.map(FieldPatch::Upsert).unwrap_or_default(),
                checkpoint_distance: req.checkpoint_distance.map(FieldPatch::Upsert).unwrap_or_default(),
                checkpoint_timeout: req.checkpoint_timeout.map(FieldPatch::Upsert).unwrap_or_default(),
                ..Default::default()
            };

            let patch_request = TenantConfigPatchRequest { tenant_id, config };
            self.pageserver_client
                .patch_tenant_config(&patch_request)
                .await
                .map_err(|error| AppError::ProjectConfigUpdateFailed {
                    reason: error.to_string(),
                })?;
        }

        self.get(user_id, org_id, id).await
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
            .map_err(|_| AppError::TenantIdInvalid {
                value: project.id.to_string(),
            })?;

        let branches = self.branch_service.list(user_id, org_id, project.id).await?;
        for branch in branches.into_iter().filter(|b| b.parent_branch_id.is_none()) {
            self.branch_service
                .delete(user_id, org_id, project.id, branch.id)
                .await?;
        }

        let mut status_code;
        loop {
            status_code = self
                .pageserver_client
                .tenant_delete(TenantShardId::unsharded(tenant_id))
                .await
                .map_err(|error| AppError::ProjectDeletionFailed {
                    reason: error.to_string(),
                })?
                .as_u16();
            if status_code != 500 && status_code != 503 && status_code != 409 {
                break;
            }
        }
        if status_code != 200 && status_code != 404 {
            return Err(AppError::ProjectDeletionFailed {
                reason: format!(
                    "Unexpected status code from pageserver when deleting tenant: {status_code}"
                ),
            });
        }

        self.project_repo.delete(project.id).await
    }

    fn validate_project_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(AppError::ProjectCreationFailed {
                reason: "Project name cannot be empty".into(),
            });
        }

        if name.len() > 255 {
            return Err(AppError::ProjectCreationFailed {
                reason: "Project name is too long (max 255 characters)".into(),
            });
        }

        Ok(())
    }
}
