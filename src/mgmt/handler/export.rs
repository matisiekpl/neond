use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::mgmt::dto::error::AppError;
use crate::mgmt::dto::export_branch_request::ExportBranchRequest;
use crate::mgmt::handler::auth::UserId;
use crate::mgmt::handler::AppState;

#[derive(Deserialize)]
pub struct DownloadQuery {
    token: Option<String>,
}

pub async fn start(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((organization_id, project_id, branch_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(req): Json<ExportBranchRequest>,
) -> Result<impl IntoResponse, AppError> {
    let status = state
        .services
        .export()
        .start(user_id, organization_id, project_id, branch_id, req)
        .await?;
    Ok((StatusCode::ACCEPTED, Json(status)))
}

pub async fn status(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((organization_id, project_id, branch_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let status = state
        .services
        .export()
        .status(user_id, organization_id, project_id, branch_id)
        .await?;
    Ok((StatusCode::OK, Json(status)))
}

pub async fn cancel(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((organization_id, project_id, branch_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let status = state
        .services
        .export()
        .cancel(user_id, organization_id, project_id, branch_id)
        .await?;
    Ok((StatusCode::OK, Json(status)))
}

pub async fn download_token(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((organization_id, project_id, branch_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let token = state
        .services
        .export()
        .create_download_token(user_id, organization_id, project_id, branch_id)
        .await?;
    Ok((StatusCode::OK, Json(token)))
}

pub async fn download(
    State(state): State<Arc<AppState>>,
    Path((organization_id, project_id, branch_id)): Path<(Uuid, Uuid, Uuid)>,
    Query(query): Query<DownloadQuery>,
) -> Result<Response, AppError> {
    let token = query.token.ok_or(AppError::Unauthorized)?;
    let download = state
        .services
        .export()
        .open_download(organization_id, project_id, branch_id, &token)
        .await?;

    let headers = [
        (header::CONTENT_TYPE, download.content_type.to_string()),
        (header::CONTENT_LENGTH, download.length.to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", download.file_name),
        ),
        (header::CACHE_CONTROL, "no-store".to_string()),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
        (header::REFERRER_POLICY, "no-referrer".to_string()),
    ];
    let body = Body::from_stream(ReaderStream::with_capacity(download.file, 256 * 1024));
    Ok((headers, body).into_response())
}
