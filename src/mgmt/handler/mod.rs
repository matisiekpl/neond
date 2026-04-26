pub mod auth;
pub mod branch;
pub mod daemon;
pub mod endpoint;
pub mod metric;
pub mod prometheus;
pub mod organization;
pub mod project;
pub mod sql;
pub mod user;

use std::sync::Arc;
use axum::{http::StatusCode, response::IntoResponse};

use crate::mgmt::dto::error::AppError;
use crate::mgmt::service::Services;

pub struct AppState {
    pub services: Arc<Services>,
}
