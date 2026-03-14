pub mod auth;
pub mod organization;
pub mod project;
pub mod user;

use axum::{http::StatusCode, response::IntoResponse};

use crate::mgmt::dto::error::AppError;
use crate::mgmt::service::Services;

pub struct AppState {
    pub services: Services,
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
