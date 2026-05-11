use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use neon_pageserver_api::models::{TimelineCreateRequest, TimelineCreateRequestMode};
use neon_utils::id::{TenantId, TimelineId};
use neon_utils::lsn::Lsn;
use neon_utils::shard::TenantShardId;
use tokio_postgres::SimpleQueryMessage;

use crate::mgmt::compute::{ComputeEndpoint, ComputeEndpointStatus};
use crate::mgmt::dto::config::Config;
use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::dto::execute_sql_request::ExecuteSqlRequest;
use crate::mgmt::dto::execute_sql_response::ExecuteSqlResponse;
use crate::mgmt::model::branch::Branch;
use crate::mgmt::model::project::{PgVersion, Project};
use crate::mgmt::repository::branch::BranchRepository;
use crate::mgmt::repository::project::ProjectRepository;
use crate::mgmt::service::endpoint::EndpointService;
use crate::mgmt::service::logs::LogsService;
use crate::mgmt::service::membership::MembershipService;
use crate::utils::password::generate_password;

pub struct SqlService {
    config: Config,
    branch_repo: Arc<BranchRepository>,
    project_repo: Arc<ProjectRepository>,
    membership_service: Arc<MembershipService>,
    endpoint_service: Arc<EndpointService>,
    pageserver_client: Arc<neon_pageserver_client::mgmt_api::Client>,
    safekeeper_client: Arc<neon_safekeeper_client::mgmt_api::Client>,
    logs_service: Arc<LogsService>,
}

impl SqlService {
    pub fn new(
        config: Config,
        branch_repo: Arc<BranchRepository>,
        project_repo: Arc<ProjectRepository>,
        membership_service: Arc<MembershipService>,
        endpoint_service: Arc<EndpointService>,
        pageserver_client: Arc<neon_pageserver_client::mgmt_api::Client>,
        safekeeper_client: Arc<neon_safekeeper_client::mgmt_api::Client>,
        logs_service: Arc<LogsService>,
    ) -> Self {
        Self {
            config,
            branch_repo,
            project_repo,
            membership_service,
            endpoint_service,
            pageserver_client,
            safekeeper_client,
            logs_service,
        }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
        request: ExecuteSqlRequest,
    ) -> Result<ExecuteSqlResponse> {
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

        match request.lsn {
            Some(lsn) => {
                self.execute_ephemeral(project, branch, lsn, request.sql)
                    .await
            }
            None => {
                self.execute_on_branch(
                    user_id,
                    organization_id,
                    project_id,
                    branch_id,
                    request.sql,
                )
                .await
            }
        }
    }

    async fn execute_on_branch(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
        sql: String,
    ) -> Result<ExecuteSqlResponse> {
        let existing = self.endpoint_service.get_endpoint_info(branch_id).await;

        if let Some(info) = &existing {
            if info.status == ComputeEndpointStatus::Running {
                return run_sql(info.port, &sql).await;
            }
        }

        let started = self
            .endpoint_service
            .start(user_id, organization_id, project_id, branch_id)
            .await?;

        run_sql(started.port, &sql).await
    }

    async fn execute_ephemeral(
        &self,
        project: Project,
        parent_branch: Branch,
        lsn: String,
        sql: String,
    ) -> Result<ExecuteSqlResponse> {
        let start_lsn = Lsn::from_str(lsn.trim()).map_err(|_| AppError::PitrLsnInvalid {
            value: lsn.clone(),
        })?;

        let ancestor_timeline_id =
            TimelineId::from_str(parent_branch.timeline_id.as_simple().to_string().as_str())
                .map_err(|_| AppError::TimelineIdInvalid {
                    value: parent_branch.timeline_id.to_string(),
                })?;

        let tenant_id = TenantId::from_str(project.id.as_simple().to_string().as_str())
            .map_err(|_| AppError::TenantIdInvalid {
                value: project.id.to_string(),
            })?;

        let new_timeline_id = TimelineId::generate();

        self.pageserver_client
            .timeline_create(
                TenantShardId::unsharded(tenant_id),
                &TimelineCreateRequest {
                    new_timeline_id,
                    mode: TimelineCreateRequestMode::Branch {
                        ancestor_timeline_id,
                        ancestor_start_lsn: Some(start_lsn),
                        pg_version: None,
                        read_only: false,
                    },
                },
            )
            .await
            .map_err(|error| AppError::EphemeralQueryFailed {
                reason: format!("Failed to create ephemeral timeline: {error}"),
            })?;

        let ephemeral_timeline_uuid = Uuid::from_str(new_timeline_id.to_string().as_str())
            .map_err(|_| AppError::TimelineIdInvalid {
                value: new_timeline_id.to_string(),
            })?;

        let ephemeral_branch = Branch {
            id: Uuid::new_v4(),
            name: "ephemeral".to_string(),
            parent_branch_id: Some(parent_branch.id),
            timeline_id: ephemeral_timeline_uuid,
            project_id: project.id,
            password: generate_password(),
            slug: format!("ephemeral-{}", Uuid::new_v4()),
            recent_status: None,
            port: None,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        };

        let mut endpoint =
            ComputeEndpoint::new(self.config.clone(), ephemeral_branch, project.pg_version.clone(), None, Arc::clone(&self.logs_service))
                .map_err(|error| AppError::ComputeStartupFailed {
                    reason: error.to_string(),
                })?;

        endpoint.launch().map_err(|error| AppError::ComputeStartupFailed {
            reason: error.to_string(),
        })?;

        let port = endpoint.get_port();
        let sql_result = run_sql(port, &sql).await;

        if let Err(e) = endpoint.shutdown() {
            tracing::warn!("Failed to shutdown ephemeral compute endpoint: {}", e);
        }

        loop {
            let status_code = match self
                .pageserver_client
                .timeline_delete(TenantShardId::unsharded(tenant_id), new_timeline_id)
                .await
            {
                Ok(code) => code.as_u16(),
                Err(e) => {
                    tracing::warn!(
                        "Failed to delete ephemeral timeline {}: {}",
                        new_timeline_id,
                        e
                    );
                    break;
                }
            };
            if status_code != 500 && status_code != 503 && status_code != 409 {
                break;
            }
        }

        if let Err(error) = self
            .safekeeper_client
            .delete_timeline(tenant_id, new_timeline_id)
            .await
        {
            tracing::warn!(
                "Failed to delete ephemeral timeline {} on safekeeper: {}",
                new_timeline_id,
                error
            );
        }

        sql_result
    }
}

pub(crate) async fn run_sql(port: u16, sql: &str) -> Result<ExecuteSqlResponse> {
    let connection_string =
        format!("host=127.0.0.1 port={port} user=cloud_admin dbname=postgres");

    let (client, connection) = tokio_postgres::connect(&connection_string, tokio_postgres::NoTls)
        .await
        .map_err(|error| AppError::SqlExecutionFailed {
            reason: format!("Failed to connect to compute endpoint: {error}"),
        })?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::warn!("Postgres connection error during SQL execution: {}", e);
        }
    });

    let messages = match client.simple_query(sql).await {
        Ok(messages) => messages,
        Err(e) => {
            let message = e
                .as_db_error()
                .map(|db| db.message().to_string())
                .unwrap_or_else(|| e.to_string());
            return Ok(ExecuteSqlResponse {
                columns: Vec::new(),
                rows: Vec::new(),
                rows_affected: None,
                error: Some(message),
            });
        }
    };

    let mut columns: Vec<String> = Vec::new();
    let mut rows: Vec<Vec<Option<String>>> = Vec::new();
    let mut rows_affected: Option<u64> = None;

    for message in messages {
        match message {
            SimpleQueryMessage::RowDescription(description) => {
                columns = description
                    .iter()
                    .map(|column| column.name().to_string())
                    .collect();
            }
            SimpleQueryMessage::Row(row) => {
                let values = (0..row.len())
                    .map(|index| row.get(index).map(|value| value.to_string()))
                    .collect();
                rows.push(values);
            }
            SimpleQueryMessage::CommandComplete(count) => {
                rows_affected = Some(count);
            }
            _ => {}
        }
    }

    Ok(ExecuteSqlResponse {
        columns,
        rows,
        rows_affected,
        error: None,
    })
}