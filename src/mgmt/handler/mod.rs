pub mod user_handler;

use std::sync::Arc;
use axum::{http::StatusCode, response::IntoResponse};

use crate::error::AppError;
use crate::mgmt::service::user_service::UserService;

pub struct AppState {
    pub user_service: UserService,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}
