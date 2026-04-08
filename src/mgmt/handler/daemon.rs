use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

use crate::mgmt::dto::error::AppError;
use crate::mgmt::dto::shutdown_daemon_request::ShutdownDaemonRequest;
use crate::mgmt::handler::auth::UserId;
use crate::mgmt::handler::AppState;

pub async fn get(
    State(state): State<Arc<AppState>>,
    UserId(_): UserId,
) -> Result<impl IntoResponse, AppError> {
    let response = state.services.daemon().get_state().await?;
    Ok((StatusCode::OK, Json(response)))
}

pub async fn shutdown(
    State(state): State<Arc<AppState>>,
    UserId(_): UserId,
    Json(req): Json<ShutdownDaemonRequest>,
) -> Result<impl IntoResponse, AppError> {
    Arc::clone(state.services.daemon())
        .request_shutdown(req.wait_for_checkpoints)
        .await?;
    Ok(StatusCode::ACCEPTED)
}

pub async fn cancel_shutdown(
    State(state): State<Arc<AppState>>,
    UserId(_): UserId,
) -> Result<impl IntoResponse, AppError> {
    state.services.daemon().cancel_shutdown().await?;
    Ok(StatusCode::NO_CONTENT)
}