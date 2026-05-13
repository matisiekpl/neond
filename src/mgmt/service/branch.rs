use crate::mgmt::compute::ComputeEndpointStatus;
use crate::mgmt::dto::branch_response::BranchResponse;
use crate::mgmt::dto::checkpoint_status::CheckpointStatus;
use crate::mgmt::dto::config::Config;
use crate::mgmt::dto::create_branch_request::CreateBranchRequest;
use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::dto::lsn_response::LsnResponse;
use crate::mgmt::dto::restore_branch_request::RestoreBranchRequest;
use crate::mgmt::dto::update_branch_request::UpdateBranchRequest;
use crate::mgmt::repository::branch::BranchRepository;
use crate::mgmt::repository::project::ProjectRepository;
use crate::mgmt::service::endpoint::EndpointService;
use crate::mgmt::service::membership::MembershipService;
use crate::utils::password::generate_password;
use names::Generator;
use neon_pageserver_api::models::{TimelineCreateRequest, TimelineCreateRequestMode};
use neon_pageserver_client::mgmt_api::ForceAwaitLogicalSize;
use neon_utils::id::{TenantId, TimelineId};
use neon_utils::lsn::Lsn;
use neon_utils::shard::TenantShardId;
use chrono::{DateTime, SecondsFormat, Utc};
use futures_util::future::join_all;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use humantime;
use reqwest;

pub struct BranchService {
    branch_repo: Arc<BranchRepository>,
    project_repo: Arc<ProjectRepository>,
    membership_service: Arc<MembershipService>,
    pageserver_client: Arc<neon_pageserver_client::mgmt_api::Client>,
    safekeeper_client: Arc<neon_safekeeper_client::mgmt_api::Client>,
    endpoint_service: Arc<EndpointService>,
    config: Config,
}

impl BranchService {
    pub fn new(
        branch_repo: Arc<BranchRepository>,
        project_repo: Arc<ProjectRepository>,
        membership_service: Arc<MembershipService>,
        pageserver_client: Arc<neon_pageserver_client::mgmt_api::Client>,
        safekeeper_client: Arc<neon_safekeeper_client::mgmt_api::Client>,
        endpoint_service: Arc<EndpointService>,
        config: Config,
    ) -> Self {
        Self {
            branch_repo,
            project_repo,
            membership_service,
            pageserver_client,
            safekeeper_client,
            endpoint_service,
            config,
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

        if self
            .branch_repo
            .find_by_project_and_name(project_id, &req.name)
            .await?
            .is_some()
        {
            return Err(AppError::BranchNameAlreadyExists {
                name: req.name.clone(),
            });
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
                    .map_err(|_| AppError::TimelineIdInvalid {
                        value: parent.timeline_id.to_string(),
                    })?;

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
            .map_err(|_| AppError::TimelineIdInvalid {
                value: new_timeline_id.to_string(),
            })?;

        let tenant_id = TenantId::from_str(project_id.as_simple().to_string().as_str())
            .map_err(|_| AppError::TenantIdInvalid {
                value: project_id.to_string(),
            })?;

        self.pageserver_client
            .timeline_create(
                TenantShardId::unsharded(tenant_id),
                &TimelineCreateRequest {
                    new_timeline_id,
                    mode,
                },
            )
            .await
            .map_err(|error| AppError::BranchCreationFailed {
                reason: error.to_string(),
            })?;

        let id = Uuid::new_v4();
        let password = generate_password();
        let slug = self.generate_unique_slug().await?;

        let branch = self
            .branch_repo
            .create(
                id,
                project_id,
                &req.name,
                req.parent_branch_id,
                timeline_uuid,
                &password,
                &slug,
            )
            .await?;

        let endpoint_info = self.endpoint_service.get_endpoint_info(branch.id).await;

        let (ancestor_timeline_id, ancestor_lsn) = self
            .fetch_ancestor(tenant_id, new_timeline_id)
            .await;

        Ok(BranchResponse {
            id: branch.id,
            project_id: branch.project_id,
            name: branch.name.clone(),
            slug: branch.slug.clone(),
            parent_branch_id: branch.parent_branch_id,
            timeline_id: branch.timeline_id,
            ancestor_timeline_id,
            ancestor_lsn,
            endpoint_status: endpoint_info
                .clone()
                .map(|info| info.status)
                .unwrap_or(ComputeEndpointStatus::Stopped),
            remote_consistent_lsn_visible: Default::default(),
            last_record_lsn: Default::default(),
            current_logical_size: 0,
            connection_string: endpoint_info
                .clone()
                .map(|info| branch.get_connection_string(self.config.clone(), info.port)),
            pooler_connection_string: endpoint_info
                .and_then(|info| info.pooler_port)
                .map(|pooler_port| {
                    branch.get_pooler_connection_string(self.config.clone(), pooler_port)
                }),
            password: branch.password.clone(),
            created_at: branch.created_at,
            updated_at: branch.updated_at,
            import_status: branch.import_status.clone(),
            import_error: branch.import_error.clone(),
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

        let tenant_id = TenantId::from_str(project_id.as_simple().to_string().as_str())
            .map_err(|_| AppError::TenantIdInvalid {
                value: project_id.to_string(),
            })?;

        let tenant_shard_id = TenantShardId::unsharded(tenant_id);

        let branch_details = join_all(branches.iter().map(|b| {
            let pageserver_client = Arc::clone(&self.pageserver_client);
            let endpoint_service = Arc::clone(&self.endpoint_service);
            let branch_id = b.id;
            let timeline_uuid = b.timeline_id;
            async move {
                let timeline_id =
                    TimelineId::from_str(timeline_uuid.as_simple().to_string().as_str())
                        .map_err(|_| AppError::TimelineIdInvalid {
                            value: timeline_uuid.to_string(),
                        })?;

                let (timeline_info, endpoint_info) = tokio::join!(
                    async {
                        pageserver_client
                            .timeline_info(
                                tenant_shard_id,
                                timeline_id,
                                ForceAwaitLogicalSize::Yes,
                            )
                            .await
                            .ok()
                    },
                    endpoint_service.get_endpoint_info(branch_id),
                );

                Ok::<_, AppError>((timeline_info, endpoint_info))
            }
        }))
        .await;

        let mut results = Vec::with_capacity(branches.len());

        for (b, details) in branches.into_iter().zip(branch_details.into_iter()) {
            let (timeline_info, endpoint_info) = details?;
            let ancestor_timeline_id = timeline_info
                .as_ref()
                .and_then(|info| info.ancestor_timeline_id)
                .and_then(|id| Uuid::from_str(id.to_string().as_str()).ok());
            results.push(BranchResponse {
                id: b.id,
                project_id: b.project_id,
                name: b.name.clone(),
                slug: b.slug.clone(),
                parent_branch_id: b.parent_branch_id,
                timeline_id: b.timeline_id,
                ancestor_timeline_id,
                ancestor_lsn: timeline_info.as_ref().and_then(|info| info.ancestor_lsn),
                endpoint_status: endpoint_info
                    .clone()
                    .map(|info| info.status)
                    .unwrap_or(ComputeEndpointStatus::Stopped),
                remote_consistent_lsn_visible: timeline_info
                    .as_ref()
                    .map(|info| info.remote_consistent_lsn_visible)
                    .unwrap_or_default(),
                last_record_lsn: timeline_info
                    .as_ref()
                    .map(|info| info.last_record_lsn)
                    .unwrap_or_default(),
                current_logical_size: timeline_info
                    .as_ref()
                    .map(|info| info.current_logical_size)
                    .unwrap_or(0),
                connection_string: endpoint_info
                    .clone()
                    .map(|info| b.get_connection_string(self.config.clone(), info.port)),
                pooler_connection_string: endpoint_info
                    .and_then(|info| info.pooler_port)
                    .map(|pooler_port| {
                        b.get_pooler_connection_string(self.config.clone(), pooler_port)
                    }),
                password: b.password.clone(),
                created_at: b.created_at,
                updated_at: b.updated_at,
                import_status: b.import_status.clone(),
                import_error: b.import_error.clone(),
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

        if let Some(existing) = self
            .branch_repo
            .find_by_project_and_name(project_id, &req.name)
            .await?
        {
            if existing.id != branch_id {
                return Err(AppError::BranchNameAlreadyExists {
                    name: req.name.clone(),
                });
            }
        }

        let updated = self.branch_repo.update(branch_id, &req.name).await?;

        let tenant_id = TenantId::from_str(project_id.as_simple().to_string().as_str())
            .map_err(|_| AppError::TenantIdInvalid {
                value: project_id.to_string(),
            })?;
        let timeline_id = TimelineId::from_str(updated.timeline_id.as_simple().to_string().as_str())
            .map_err(|_| AppError::TimelineIdInvalid {
                value: updated.timeline_id.to_string(),
            })?;
        let (ancestor_timeline_id, ancestor_lsn) =
            self.fetch_ancestor(tenant_id, timeline_id).await;

        let endpoint_info = self.endpoint_service.get_endpoint_info(branch.id).await;
        Ok(BranchResponse {
            id: updated.id,
            project_id: updated.project_id,
            name: updated.name.clone(),
            slug: updated.slug.clone(),
            parent_branch_id: updated.parent_branch_id,
            timeline_id: updated.timeline_id,
            ancestor_timeline_id,
            ancestor_lsn,
            endpoint_status: endpoint_info
                .clone()
                .map(|info| info.status)
                .unwrap_or(ComputeEndpointStatus::Stopped),
            remote_consistent_lsn_visible: Default::default(),
            last_record_lsn: Default::default(),
            current_logical_size: 0,
            connection_string: endpoint_info
                .as_ref()
                .map(|info| branch.get_connection_string(self.config.clone(), info.port)),
            pooler_connection_string: endpoint_info
                .as_ref()
                .and_then(|info| info.pooler_port)
                .map(|pooler_port| {
                    branch.get_pooler_connection_string(self.config.clone(), pooler_port)
                }),
            password: updated.password.clone(),
            created_at: updated.created_at,
            updated_at: updated.updated_at,
            import_status: updated.import_status.clone(),
            import_error: updated.import_error.clone(),
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
            .map_err(|_| AppError::TenantIdInvalid {
                value: project_id.to_string(),
            })?;

        let mut to_delete: Vec<Uuid> = Vec::new();
        let mut stack = vec![branch_id];

        while let Some(id) = stack.pop() {
            let children = self.branch_repo.list_by_parent_id(id).await?;
            for child in children {
                stack.push(child.id);
            }
            to_delete.push(id);
        }

        to_delete.reverse();

        for id in to_delete {
            let branch = self
                .branch_repo
                .find_by_id(id)
                .await?
                .ok_or(AppError::NotFound)?;

            let _ = self
                .endpoint_service
                .stop(user_id, org_id, project_id, id)
                .await;

            let timeline_id =
                TimelineId::from_str(branch.timeline_id.as_simple().to_string().as_str())
                    .map_err(|_| AppError::TimelineIdInvalid {
                        value: branch.timeline_id.to_string(),
                    })?;

            let mut status_code;
            loop {
                status_code = self
                    .pageserver_client
                    .timeline_delete(TenantShardId::unsharded(tenant_id), timeline_id)
                    .await
                    .map_err(|error| AppError::BranchDeletionFailed {
                        reason: error.to_string(),
                    })?
                    .as_u16();
                if status_code != 500 && status_code != 503 && status_code != 409 {
                    break;
                }
            }

            if status_code != 200 && status_code != 404 {
                return Err(AppError::BranchDeletionFailed {
                    reason: format!("Unexpected status code from pageserver: {status_code}"),
                });
            }

            if let Err(error) = self
                .safekeeper_client
                .delete_timeline(tenant_id, timeline_id)
                .await
            {
                return Err(AppError::BranchDeletionFailed {
                    reason: format!("safekeeper timeline delete failed: {error}"),
                });
            }

            self.branch_repo.delete(id).await?;
        }

        Ok(())
    }

    pub async fn lsn(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
        timestamp: DateTime<Utc>,
    ) -> Result<LsnResponse> {
        self.membership_service
            .verify_membership(user_id, organization_id)
            .await?;

        let project = self
            .project_repo
            .find_by_id(project_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if project.organization_id != organization_id {
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
            .map_err(|_| AppError::TenantIdInvalid {
                value: project_id.to_string(),
            })?;
        let tenant_shard_id = TenantShardId::unsharded(tenant_id);

        let timeline_id = TimelineId::from_str(branch.timeline_id.as_simple().to_string().as_str())
            .map_err(|_| AppError::TimelineIdInvalid {
                value: branch.timeline_id.to_string(),
            })?;

        let token = self
            .config
            .component_auth
            .generate_token(neon_utils::auth::Scope::PageServerApi, None)?;

        let timestamp_string = timestamp.to_rfc3339_opts(SecondsFormat::Millis, true);

        let response = reqwest::Client::new()
            .get(format!(
                "http://127.0.0.1:1234/v1/tenant/{tenant_shard_id}/timeline/{timeline_id}/get_lsn_by_timestamp"
            ))
            .query(&[("timestamp", &timestamp_string)])
            .bearer_auth(token)
            .send()
            .await
            .map_err(|error| AppError::PageserverApiFailed {
                operation: "get_lsn_by_timestamp".to_string(),
                reason: error.to_string(),
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::PageserverApiFailed {
                operation: "get_lsn_by_timestamp".to_string(),
                reason: format!("Pageserver returned {status}: {body}"),
            });
        }

        response
            .json::<LsnResponse>()
            .await
            .map_err(|error| AppError::PageserverApiFailed {
                operation: "get_lsn_by_timestamp".to_string(),
                reason: format!("Invalid pageserver response: {error}"),
            })
    }

    pub async fn restore(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
        request: RestoreBranchRequest,
    ) -> Result<BranchResponse> {
        self.membership_service
            .verify_membership(user_id, organization_id)
            .await?;

        let project = self
            .project_repo
            .find_by_id(project_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if project.organization_id != organization_id {
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

        let target_lsn = Lsn::from_str(request.lsn.trim()).map_err(|_| {
            AppError::PitrLsnInvalid {
                value: request.lsn.clone(),
            }
        })?;

        let endpoint_info = self.endpoint_service.get_endpoint_info(branch.id).await;

        if let Some(ref info) = endpoint_info {
            match info.status {
                ComputeEndpointStatus::Starting | ComputeEndpointStatus::Stopping => {
                    return Err(AppError::PitrConcurrentEndpointOperation);
                }
                _ => {}
            }
        }

        let was_running = endpoint_info
            .as_ref()
            .map(|info| info.status == ComputeEndpointStatus::Running)
            .unwrap_or(false);

        if was_running {
            self.endpoint_service
                .stop(user_id, organization_id, project_id, branch_id)
                .await?;
        }

        let ancestor_timeline_id =
            TimelineId::from_str(branch.timeline_id.as_simple().to_string().as_str())
                .map_err(|_| AppError::TimelineIdInvalid {
                    value: branch.timeline_id.to_string(),
                })?;

        let tenant_id = TenantId::from_str(project_id.as_simple().to_string().as_str())
            .map_err(|_| AppError::TenantIdInvalid {
                value: project_id.to_string(),
            })?;
        let tenant_shard_id = TenantShardId::unsharded(tenant_id);

        let new_timeline_id = TimelineId::generate();

        self.pageserver_client
            .timeline_create(
                tenant_shard_id,
                &TimelineCreateRequest {
                    new_timeline_id,
                    mode: TimelineCreateRequestMode::Branch {
                        ancestor_timeline_id,
                        ancestor_start_lsn: Some(target_lsn),
                        pg_version: None,
                        read_only: false,
                    },
                },
            )
            .await
            .map_err(|error| {
                let message = error.to_string();
                let lower = message.to_lowercase();
                if lower.contains("lsn")
                    || lower.contains("bad request")
                    || lower.contains("out of range")
                    || lower.contains("not found")
                {
                    AppError::PitrLsnOutOfRange { reason: message }
                } else {
                    AppError::PitrTimelineCreationFailed { reason: message }
                }
            })?;

        let detached = match self
            .pageserver_client
            .timeline_detach_ancestor(tenant_shard_id, new_timeline_id, None)
            .await
        {
            Ok(detached) => detached,
            Err(error) => {
                if let Err(cleanup_error) = self
                    .pageserver_client
                    .timeline_delete(tenant_shard_id, new_timeline_id)
                    .await
                {
                    tracing::warn!(
                        "Failed to clean up orphan timeline {} after detach_ancestor failure: {}",
                        new_timeline_id,
                        cleanup_error
                    );
                }
                if let Err(cleanup_error) = self
                    .safekeeper_client
                    .delete_timeline(tenant_id, new_timeline_id)
                    .await
                {
                    tracing::warn!(
                        "Failed to clean up orphan timeline {} on safekeeper after detach_ancestor failure: {}",
                        new_timeline_id,
                        cleanup_error
                    );
                }
                return Err(AppError::DetachAncestorFailed {
                    reason: error.to_string(),
                });
            }
        };

        let reparented_timeline_ids: HashSet<Uuid> = detached
            .reparented_timelines
            .iter()
            .filter_map(|id| Uuid::from_str(id.to_string().as_str()).ok())
            .collect();

        let new_timeline_uuid = Uuid::from_str(new_timeline_id.to_string().as_str())
            .map_err(|_| AppError::TimelineIdInvalid {
                value: new_timeline_id.to_string(),
            })?;

        let archive_slug = self.generate_unique_slug().await?;
        let archive_name = format!(
            "{}_pitr_archived_{}",
            branch.name,
            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
        );

        let new_branch_id = Uuid::new_v4();

        let inserted = match self
            .branch_repo
            .restore_swap(
                branch.id,
                &archive_slug,
                &archive_name,
                new_branch_id,
                &branch.slug,
                &branch.password,
                &branch.name,
                new_timeline_uuid,
                branch.project_id,
                &reparented_timeline_ids,
                branch.port,
            )
            .await
        {
            Ok(inserted) => inserted,
            Err(error) => {
                if let Err(cleanup_error) = self
                    .pageserver_client
                    .timeline_delete(tenant_shard_id, new_timeline_id)
                    .await
                {
                    tracing::warn!(
                        "Failed to clean up orphan timeline {} after PITR swap failure: {}",
                        new_timeline_id,
                        cleanup_error
                    );
                }
                if let Err(cleanup_error) = self
                    .safekeeper_client
                    .delete_timeline(tenant_id, new_timeline_id)
                    .await
                {
                    tracing::warn!(
                        "Failed to clean up orphan timeline {} on safekeeper after PITR swap failure: {}",
                        new_timeline_id,
                        cleanup_error
                    );
                }
                return Err(error);
            }
        };

        if was_running {
            if let Err(error) = self
                .endpoint_service
                .start(user_id, organization_id, project_id, inserted.id)
                .await
            {
                return Err(AppError::PitrEndpointRelaunchFailed {
                    reason: error.to_string(),
                });
            }
        }

        let endpoint_info = self.endpoint_service.get_endpoint_info(inserted.id).await;

        Ok(BranchResponse {
            id: inserted.id,
            project_id: inserted.project_id,
            name: inserted.name.clone(),
            slug: inserted.slug.clone(),
            parent_branch_id: inserted.parent_branch_id,
            timeline_id: inserted.timeline_id,
            ancestor_timeline_id: None,
            ancestor_lsn: None,
            endpoint_status: endpoint_info
                .clone()
                .map(|info| info.status)
                .unwrap_or(ComputeEndpointStatus::Stopped),
            remote_consistent_lsn_visible: Default::default(),
            last_record_lsn: Default::default(),
            current_logical_size: 0,
            connection_string: endpoint_info
                .clone()
                .map(|info| inserted.get_connection_string(self.config.clone(), info.port)),
            pooler_connection_string: endpoint_info
                .and_then(|info| info.pooler_port)
                .map(|pooler_port| {
                    inserted.get_pooler_connection_string(self.config.clone(), pooler_port)
                }),
            password: inserted.password.clone(),
            created_at: inserted.created_at,
            updated_at: inserted.updated_at,
            import_status: inserted.import_status.clone(),
            import_error: inserted.import_error.clone(),
        })
    }

    pub async fn reset_to_parent(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
    ) -> Result<BranchResponse> {
        self.membership_service
            .verify_membership(user_id, organization_id)
            .await?;

        let project = self
            .project_repo
            .find_by_id(project_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if project.organization_id != organization_id {
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

        let parent_branch_id = branch.parent_branch_id.ok_or(AppError::BranchUpdateFailed {
            reason: "Branch has no parent".into(),
        })?;

        let parent = self
            .branch_repo
            .find_by_id(parent_branch_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if parent.project_id != project_id {
            return Err(AppError::NotFound);
        }

        let has_children = !self
            .branch_repo
            .list_by_parent_id(branch.id)
            .await?
            .is_empty();

        let endpoint_info = self.endpoint_service.get_endpoint_info(branch.id).await;

        if let Some(ref info) = endpoint_info {
            match info.status {
                ComputeEndpointStatus::Starting | ComputeEndpointStatus::Stopping => {
                    return Err(AppError::PitrConcurrentEndpointOperation);
                }
                _ => {}
            }
        }

        let was_running = endpoint_info
            .as_ref()
            .map(|info| info.status == ComputeEndpointStatus::Running)
            .unwrap_or(false);

        if was_running {
            self.endpoint_service
                .stop(user_id, organization_id, project_id, branch_id)
                .await?;
        }

        let ancestor_timeline_id =
            TimelineId::from_str(parent.timeline_id.as_simple().to_string().as_str())
                .map_err(|_| AppError::TimelineIdInvalid {
                    value: parent.timeline_id.to_string(),
                })?;

        let tenant_id = TenantId::from_str(project_id.as_simple().to_string().as_str())
            .map_err(|_| AppError::TenantIdInvalid {
                value: project_id.to_string(),
            })?;
        let tenant_shard_id = TenantShardId::unsharded(tenant_id);

        let new_timeline_id = TimelineId::generate();

        self.pageserver_client
            .timeline_create(
                tenant_shard_id,
                &TimelineCreateRequest {
                    new_timeline_id,
                    mode: TimelineCreateRequestMode::Branch {
                        ancestor_timeline_id,
                        ancestor_start_lsn: None,
                        pg_version: None,
                        read_only: false,
                    },
                },
            )
            .await
            .map_err(|error| AppError::PitrTimelineCreationFailed {
                reason: error.to_string(),
            })?;

        let new_timeline_uuid = Uuid::from_str(new_timeline_id.to_string().as_str())
            .map_err(|_| AppError::TimelineIdInvalid {
                value: new_timeline_id.to_string(),
            })?;

        let archive_slug = self.generate_unique_slug().await?;
        let archive_name = format!(
            "{}_reset_archived_{}",
            branch.name,
            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
        );

        let new_branch_id = Uuid::new_v4();

        let inserted = match self
            .branch_repo
            .reset_to_parent_swap(
                branch.id,
                &archive_slug,
                &archive_name,
                new_branch_id,
                &branch.slug,
                &branch.password,
                &branch.name,
                new_timeline_uuid,
                parent.id,
                branch.project_id,
                branch.port,
            )
            .await
        {
            Ok(inserted) => inserted,
            Err(error) => {
                if let Err(cleanup_error) = self
                    .pageserver_client
                    .timeline_delete(tenant_shard_id, new_timeline_id)
                    .await
                {
                    tracing::warn!(
                        "Failed to clean up orphan timeline {} after reset-to-parent swap failure: {}",
                        new_timeline_id,
                        cleanup_error
                    );
                }
                if let Err(cleanup_error) = self
                    .safekeeper_client
                    .delete_timeline(tenant_id, new_timeline_id)
                    .await
                {
                    tracing::warn!(
                        "Failed to clean up orphan timeline {} on safekeeper after reset-to-parent swap failure: {}",
                        new_timeline_id,
                        cleanup_error
                    );
                }
                return Err(error);
            }
        };

        if was_running {
            if let Err(error) = self
                .endpoint_service
                .start(user_id, organization_id, project_id, inserted.id)
                .await
            {
                return Err(AppError::PitrEndpointRelaunchFailed {
                    reason: error.to_string(),
                });
            }
        }

        if !has_children {
            if let Err(error) = self
                .delete(user_id, organization_id, project_id, branch.id)
                .await
            {
                tracing::warn!(
                    "Failed to prune archived branch {} after reset-to-parent: {}",
                    branch.id,
                    error
                );
            }
        }

        let endpoint_info = self.endpoint_service.get_endpoint_info(inserted.id).await;

        let (ancestor_timeline_id, ancestor_lsn) = self
            .fetch_ancestor(tenant_id, new_timeline_id)
            .await;

        Ok(BranchResponse {
            id: inserted.id,
            project_id: inserted.project_id,
            name: inserted.name.clone(),
            slug: inserted.slug.clone(),
            parent_branch_id: inserted.parent_branch_id,
            timeline_id: inserted.timeline_id,
            ancestor_timeline_id,
            ancestor_lsn,
            endpoint_status: endpoint_info
                .clone()
                .map(|info| info.status)
                .unwrap_or(ComputeEndpointStatus::Stopped),
            remote_consistent_lsn_visible: Default::default(),
            last_record_lsn: Default::default(),
            current_logical_size: 0,
            connection_string: endpoint_info
                .clone()
                .map(|info| inserted.get_connection_string(self.config.clone(), info.port)),
            pooler_connection_string: endpoint_info
                .and_then(|info| info.pooler_port)
                .map(|pooler_port| {
                    inserted.get_pooler_connection_string(self.config.clone(), pooler_port)
                }),
            password: inserted.password.clone(),
            created_at: inserted.created_at,
            updated_at: inserted.updated_at,
            import_status: inserted.import_status.clone(),
            import_error: inserted.import_error.clone(),
        })
    }

    pub async fn detach_ancestor(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
    ) -> Result<BranchResponse> {
        self.membership_service
            .verify_membership(user_id, organization_id)
            .await?;

        let project = self
            .project_repo
            .find_by_id(project_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if project.organization_id != organization_id {
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

        if branch.parent_branch_id.is_none() {
            return Err(AppError::BranchAlreadyDetached);
        }

        let timeline_id =
            TimelineId::from_str(branch.timeline_id.as_simple().to_string().as_str())
                .map_err(|_| AppError::TimelineIdInvalid {
                    value: branch.timeline_id.to_string(),
                })?;

        let tenant_id = TenantId::from_str(project_id.as_simple().to_string().as_str())
            .map_err(|_| AppError::TenantIdInvalid {
                value: project_id.to_string(),
            })?;
        let tenant_shard_id = TenantShardId::unsharded(tenant_id);

        let detached = self
            .pageserver_client
            .timeline_detach_ancestor(tenant_shard_id, timeline_id, None)
            .await
            .map_err(|error| AppError::DetachAncestorFailed {
                reason: error.to_string(),
            })?;

        let reparented_timeline_ids: HashSet<Uuid> = detached
            .reparented_timelines
            .iter()
            .filter_map(|id| Uuid::from_str(id.to_string().as_str()).ok())
            .collect();

        let updated = self
            .branch_repo
            .detach_ancestor_swap(branch.id, &reparented_timeline_ids)
            .await?;

        let endpoint_info = self.endpoint_service.get_endpoint_info(updated.id).await;

        let (ancestor_timeline_id, ancestor_lsn) =
            self.fetch_ancestor(tenant_id, timeline_id).await;

        Ok(BranchResponse {
            id: updated.id,
            project_id: updated.project_id,
            name: updated.name.clone(),
            slug: updated.slug.clone(),
            parent_branch_id: updated.parent_branch_id,
            timeline_id: updated.timeline_id,
            ancestor_timeline_id,
            ancestor_lsn,
            endpoint_status: endpoint_info
                .clone()
                .map(|info| info.status)
                .unwrap_or(ComputeEndpointStatus::Stopped),
            remote_consistent_lsn_visible: Default::default(),
            last_record_lsn: Default::default(),
            current_logical_size: 0,
            connection_string: endpoint_info
                .clone()
                .map(|info| updated.get_connection_string(self.config.clone(), info.port)),
            pooler_connection_string: endpoint_info
                .and_then(|info| info.pooler_port)
                .map(|pooler_port| {
                    updated.get_pooler_connection_string(self.config.clone(), pooler_port)
                }),
            password: updated.password.clone(),
            created_at: updated.created_at,
            updated_at: updated.updated_at,
            import_status: updated.import_status.clone(),
            import_error: updated.import_error.clone(),
        })
    }

    pub async fn change_password(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
        password: String,
    ) -> Result<BranchResponse> {
        if password.trim().is_empty() {
            return Err(AppError::BranchUpdateFailed {
                reason: "Password cannot be empty".into(),
            });
        }

        self.membership_service
            .verify_membership(user_id, organization_id)
            .await?;

        let project = self
            .project_repo
            .find_by_id(project_id)
            .await?
            .ok_or(AppError::NotFound)?;

        if project.organization_id != organization_id {
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

        let endpoint_info = self.endpoint_service.get_endpoint_info(branch.id).await;

        if let Some(ref info) = endpoint_info {
            match info.status {
                ComputeEndpointStatus::Starting | ComputeEndpointStatus::Stopping => {
                    return Err(AppError::PitrConcurrentEndpointOperation);
                }
                _ => {}
            }
        }

        let was_running = endpoint_info
            .as_ref()
            .map(|info| info.status == ComputeEndpointStatus::Running)
            .unwrap_or(false);

        let updated = self
            .branch_repo
            .update_password(branch_id, &password)
            .await?;

        if was_running {
            self.endpoint_service
                .stop(user_id, organization_id, project_id, branch_id)
                .await?;
            self.endpoint_service
                .start(user_id, organization_id, project_id, branch_id)
                .await?;
        }

        let tenant_id = TenantId::from_str(project_id.as_simple().to_string().as_str())
            .map_err(|_| AppError::TenantIdInvalid {
                value: project_id.to_string(),
            })?;
        let timeline_id = TimelineId::from_str(updated.timeline_id.as_simple().to_string().as_str())
            .map_err(|_| AppError::TimelineIdInvalid {
                value: updated.timeline_id.to_string(),
            })?;
        let (ancestor_timeline_id, ancestor_lsn) =
            self.fetch_ancestor(tenant_id, timeline_id).await;

        let endpoint_info = self.endpoint_service.get_endpoint_info(updated.id).await;

        Ok(BranchResponse {
            id: updated.id,
            project_id: updated.project_id,
            name: updated.name.clone(),
            slug: updated.slug.clone(),
            parent_branch_id: updated.parent_branch_id,
            timeline_id: updated.timeline_id,
            ancestor_timeline_id,
            ancestor_lsn,
            endpoint_status: endpoint_info
                .clone()
                .map(|info| info.status)
                .unwrap_or(ComputeEndpointStatus::Stopped),
            remote_consistent_lsn_visible: Default::default(),
            last_record_lsn: Default::default(),
            current_logical_size: 0,
            connection_string: endpoint_info
                .clone()
                .map(|info| updated.get_connection_string(self.config.clone(), info.port)),
            pooler_connection_string: endpoint_info
                .and_then(|info| info.pooler_port)
                .map(|pooler_port| {
                    updated.get_pooler_connection_string(self.config.clone(), pooler_port)
                }),
            password: updated.password.clone(),
            created_at: updated.created_at,
            updated_at: updated.updated_at,
            import_status: updated.import_status.clone(),
            import_error: updated.import_error.clone(),
        })
    }

    pub async fn check_branches_durability(&self) -> Result<CheckpointStatus> {
        let projects = self.project_repo.list_all().await?;
        let mut all_in_sync = true;
        let mut max_checkpoint_timeout: Option<Duration> = None;

        for project in projects {
            let tenant_id =
                TenantId::from_str(project.id.as_simple().to_string().as_str())
                    .map_err(|_| AppError::TenantIdInvalid {
                        value: project.id.to_string(),
                    })?;
            let tenant_shard_id = TenantShardId::unsharded(tenant_id);

            let token = self
                .config
                .component_auth
                .generate_token(neon_utils::auth::Scope::PageServerApi, None)?;
            let config_response = reqwest::Client::new()
                .get(format!(
                    "http://127.0.0.1:1234/v1/tenant/{tenant_shard_id}/config"
                ))
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await
                .ok();

            if let Some(response) = config_response {
                let value: serde_json::Value = response.json().await.unwrap_or_default();
                let overrides = value
                    .get("tenant_specific_overrides")
                    .cloned()
                    .unwrap_or_default();

                if let Some(timeout_str) = overrides
                    .get("checkpoint_timeout")
                    .and_then(|v| v.as_str())
                {
                    if let Ok(duration) = humantime::parse_duration(timeout_str) {
                        max_checkpoint_timeout = Some(
                            max_checkpoint_timeout
                                .map_or(duration, |current| current.max(duration)),
                        );
                    }
                }
            }

            let branches = self.branch_repo.list_by_project_id(project.id).await?;
            for branch in branches {
                let timeline_id =
                    TimelineId::from_str(branch.timeline_id.as_simple().to_string().as_str())
                        .map_err(|_| AppError::TimelineIdInvalid {
                            value: branch.timeline_id.to_string(),
                        })?;

                let timeline_info = self
                    .pageserver_client
                    .timeline_info(
                        tenant_shard_id,
                        timeline_id,
                        ForceAwaitLogicalSize::Yes,
                    )
                    .await
                    .map_err(|error| AppError::DurabilityCheckFailed {
                        reason: error.to_string(),
                    })?;

                if timeline_info.remote_consistent_lsn_visible != timeline_info.last_record_lsn {
                    all_in_sync = false;
                }
            }
        }

        Ok(CheckpointStatus {
            all_in_sync,
            max_checkpoint_timeout,
        })
    }

    async fn fetch_ancestor(
        &self,
        tenant_id: TenantId,
        timeline_id: TimelineId,
    ) -> (Option<Uuid>, Option<Lsn>) {
        match self
            .pageserver_client
            .timeline_info(
                TenantShardId::unsharded(tenant_id),
                timeline_id,
                ForceAwaitLogicalSize::No,
            )
            .await
        {
            Ok(info) => (
                info.ancestor_timeline_id
                    .and_then(|id| Uuid::from_str(id.to_string().as_str()).ok()),
                info.ancestor_lsn,
            ),
            Err(_) => (None, None),
        }
    }

    pub async fn generate_unique_slug(&self) -> Result<String> {
        for _ in 0..10 {
            let slug = Generator::default()
                .next()
                .unwrap_or_else(|| format!("branch-{}", Uuid::new_v4()));

            if self.branch_repo.find_by_slug(&slug).await?.is_none() {
                return Ok(slug);
            }
        }

        Ok(format!("branch-{}", Uuid::new_v4()))
    }

    pub fn validate_branch_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(AppError::BranchCreationFailed {
                reason: "Branch name cannot be empty".into(),
            });
        }

        if name.len() > 255 {
            return Err(AppError::BranchCreationFailed {
                reason: "Branch name is too long (max 255 characters)".into(),
            });
        }

        Ok(())
    }
}
