use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::mgmt::dto::error::AppError;
use crate::mgmt::dto::metric_range_request::MetricRangeRequest;
use crate::mgmt::dto::metric_sample::MetricSample;
use crate::mgmt::handler::auth::UserId;
use crate::mgmt::handler::AppState;

pub async fn list_for_branch(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path((organization_id, project_id, branch_id)): Path<(Uuid, Uuid, Uuid)>,
    Query(range): Query<MetricRangeRequest>,
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now().naive_utc();
    let to = range.to.unwrap_or(now);
    let from = range.from.unwrap_or(to - chrono::Duration::hours(1));

    let samples = state
        .services
        .metric()
        .list(user_id, organization_id, project_id, branch_id, from, to)
        .await?;

    let response: Vec<MetricSample> = samples
        .into_iter()
        .map(|sample| MetricSample {
            recorded_at: sample.recorded_at,
            slug: sample.slug,
            value: sample.value,
        })
        .collect();

    Ok((StatusCode::OK, Json(response)))
}

pub async fn list_daemon(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Path(organization_id): Path<Uuid>,
    Query(range): Query<MetricRangeRequest>,
) -> Result<impl IntoResponse, AppError> {
    let now = Utc::now().naive_utc();
    let to = range.to.unwrap_or(now);
    let from = range.from.unwrap_or(to - chrono::Duration::hours(1));

    let samples = state
        .services
        .metric()
        .list_daemon(user_id, organization_id, from, to)
        .await?;

    let response: Vec<MetricSample> = samples
        .into_iter()
        .map(|sample| MetricSample {
            recorded_at: sample.recorded_at,
            slug: sample.slug,
            value: sample.value,
        })
        .collect();

    Ok((StatusCode::OK, Json(response)))
}