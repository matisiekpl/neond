use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::mgmt::dto::error::AppError;
use crate::mgmt::handler::auth::UserId;
use crate::mgmt::handler::AppState;

pub async fn start(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((org_id, project_id, branch_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let endpoint = state
        .services
        .endpoint()
        .start(user_id, org_id, project_id, branch_id)
        .await?;
    Ok((StatusCode::OK, Json(endpoint)))
}

pub async fn stop(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((org_id, project_id, branch_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let endpoint = state
        .services
        .endpoint()
        .stop(user_id, org_id, project_id, branch_id)
        .await?;
    Ok((StatusCode::OK, Json(endpoint)))
}

pub async fn status(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((org_id, project_id, branch_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let endpoint = state
        .services
        .endpoint()
        .status(user_id, org_id, project_id, branch_id)
        .await?;
    Ok((StatusCode::OK, Json(endpoint)))
}
