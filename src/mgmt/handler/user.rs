use std::sync::Arc;
use axum::{extract::{Path, State}, http::{HeaderMap, StatusCode}, response::IntoResponse, Json};
use uuid::Uuid;
use crate::mgmt::dto::change_password_request::ChangePasswordRequest;
use crate::mgmt::dto::create_user_request::CreateUserRequest;
use crate::mgmt::dto::error::AppError;
use crate::mgmt::dto::login_user_request::LoginUserRequest;
use crate::mgmt::dto::register_user_request::RegisterUserRequest;
use crate::mgmt::dto::update_user_request::UpdateUserRequest;
use crate::mgmt::handler::auth::UserId;
use crate::mgmt::handler::AppState;

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let res = state.services.user().login(req).await?;
    Ok((StatusCode::OK, Json(res)))
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let res = state.services.user().register(req).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

pub async fn setup(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let res = state.services.user().setup().await?;
    Ok((StatusCode::OK, Json(res)))
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;
    let res = state.services.user().me(token).await?;
    Ok((StatusCode::OK, Json(res)))
}

pub async fn list_users(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> Result<impl IntoResponse, AppError> {
    let res = state.services.user().list_users(user_id).await?;
    Ok((StatusCode::OK, Json(res)))
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Json(req): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let res = state.services.user().create_user(user_id, req).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path(target_user_id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let res = state.services.user().update_user(user_id, target_user_id, req).await?;
    Ok((StatusCode::OK, Json(res)))
}

pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path(target_user_id): Path<Uuid>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    state.services.user().reset_password(user_id, target_user_id, req.password).await?;
    Ok((StatusCode::NO_CONTENT, ()))
}

pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path(target_user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    state.services.user().delete_user(user_id, target_user_id).await?;
    Ok((StatusCode::NO_CONTENT, ()))
}
