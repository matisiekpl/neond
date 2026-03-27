use std::sync::Arc;
use axum::{extract::State, http::{HeaderMap, StatusCode}, response::IntoResponse, Json};
use crate::mgmt::dto::error::AppError;
use crate::mgmt::dto::login_user_request::LoginUserRequest;
use crate::mgmt::dto::register_user_request::RegisterUserRequest;
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
