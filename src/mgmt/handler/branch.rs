use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::mgmt::dto::error::AppError;
use crate::mgmt::dto::create_branch_request::CreateBranchRequest;
use crate::mgmt::dto::update_branch_request::UpdateBranchRequest;
use crate::mgmt::handler::auth::UserId;
use crate::mgmt::handler::AppState;

pub async fn create(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((org_id, project_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<CreateBranchRequest>,
) -> Result<impl IntoResponse, AppError> {
    let branch = state.services.branch().create(user_id, org_id, project_id, req).await?;
    Ok((StatusCode::CREATED, Json(branch)))
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((org_id, project_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let branches = state.services.branch().list(user_id, org_id, project_id).await?;
    Ok((StatusCode::OK, Json(branches)))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((org_id, project_id, branch_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(req): Json<UpdateBranchRequest>,
) -> Result<impl IntoResponse, AppError> {
    let branch = state.services.branch().update(user_id, org_id, project_id, branch_id, req).await?;
    Ok((StatusCode::OK, Json(branch)))
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((org_id, project_id, branch_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    state.services.branch().delete(user_id, org_id, project_id, branch_id).await?;
    Ok((StatusCode::NO_CONTENT, ()))
}
