use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

use crate::mgmt::dto::error::AppError;
use crate::mgmt::handler::auth::UserId;
use crate::mgmt::handler::AppState;

pub async fn get(
    State(state): State<Arc<AppState>>,
    UserId(_): UserId,
) -> Result<impl IntoResponse, AppError> {
    let response = state.services.daemon().get_state().await?;
    Ok((StatusCode::OK, Json(response)))
}