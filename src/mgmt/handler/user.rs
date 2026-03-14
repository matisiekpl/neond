use std::sync::Arc;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

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
