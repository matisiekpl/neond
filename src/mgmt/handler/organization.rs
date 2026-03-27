use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::mgmt::dto::add_member_by_email_request::AddMemberByEmailRequest;
use crate::mgmt::dto::error::AppError;
use crate::mgmt::dto::create_organization_request::CreateOrganizationRequest;
use crate::mgmt::dto::update_organization_request::UpdateOrganizationRequest;
use crate::mgmt::handler::auth::UserId;
use crate::mgmt::handler::AppState;

pub async fn create(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Json(req): Json<CreateOrganizationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let org = state.services.organization().create(user_id, req).await?;
    Ok((StatusCode::CREATED, Json(org)))
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let org = state.services.organization().get(user_id, org_id).await?;
    Ok((StatusCode::OK, Json(org)))
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> Result<impl IntoResponse, AppError> {
    let orgs = state.services.organization().list(user_id).await?;
    Ok((StatusCode::OK, Json(orgs)))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path(org_id): Path<Uuid>,
    Json(req): Json<UpdateOrganizationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let org = state.services.organization().update(user_id, org_id, req).await?;
    Ok((StatusCode::OK, Json(org)))
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    state.services.organization().delete(user_id, org_id).await?;
    Ok((StatusCode::NO_CONTENT, ()))
}

pub async fn assign_member(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((org_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    state.services.organization().assign_member(user_id, org_id, target_user_id).await?;
    Ok((StatusCode::CREATED, ()))
}

pub async fn revoke_member(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((org_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    state.services.organization().revoke_member(user_id, org_id, target_user_id).await?;
    Ok((StatusCode::NO_CONTENT, ()))
}

pub async fn list_members(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let members = state.services.organization().list_members(user_id, org_id).await?;
    Ok((StatusCode::OK, Json(members)))
}

pub async fn assign_member_by_email(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path(org_id): Path<Uuid>,
    Json(req): Json<AddMemberByEmailRequest>,
) -> Result<impl IntoResponse, AppError> {
    state
        .services
        .organization()
        .assign_member_by_email(user_id, org_id, &req.email)
        .await?;
    Ok((StatusCode::CREATED, ()))
}
