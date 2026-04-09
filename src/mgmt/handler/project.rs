use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::mgmt::dto::error::AppError;
use crate::mgmt::dto::create_project_request::CreateProjectRequest;
use crate::mgmt::dto::update_project_request::UpdateProjectRequest;
use crate::mgmt::handler::auth::UserId;
use crate::mgmt::handler::AppState;

pub async fn create(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path(organization_id): Path<Uuid>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<impl IntoResponse, AppError> {
    let project = state.services.project().create(user_id, organization_id, req).await?;
    Ok((StatusCode::CREATED, Json(project)))
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((organization_id, project_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let project = state.services.project().get(user_id, organization_id, project_id).await?;
    Ok((StatusCode::OK, Json(project)))
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path(organization_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let projects = state.services.project().list(user_id, organization_id).await?;
    Ok((StatusCode::OK, Json(projects)))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((organization_id, project_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<impl IntoResponse, AppError> {
    let project = state.services.project().update(user_id, organization_id, project_id, req).await?;
    Ok((StatusCode::OK, Json(project)))
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((organization_id, project_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    state.services.project().delete(user_id, organization_id, project_id).await?;
    Ok((StatusCode::NO_CONTENT, ()))
}
