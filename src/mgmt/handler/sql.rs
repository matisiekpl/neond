use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::mgmt::dto::error::AppError;
use crate::mgmt::dto::execute_sql_request::ExecuteSqlRequest;
use crate::mgmt::handler::auth::UserId;
use crate::mgmt::handler::AppState;

pub async fn execute(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((organization_id, project_id, branch_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(request): Json<ExecuteSqlRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = state
        .services
        .sql()
        .execute(user_id, organization_id, project_id, branch_id, request)
        .await?;
    Ok((StatusCode::OK, Json(response)))
}