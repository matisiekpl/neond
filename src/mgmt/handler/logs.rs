use axum::extract::{Path, Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::{self, Stream, StreamExt};
use serde::Deserialize;
use std::convert::Infallible;
use std::str::FromStr;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

use crate::mgmt::dto::error::AppError;
use crate::mgmt::handler::auth::{UserId, authenticate};
use crate::mgmt::handler::AppState;
use crate::mgmt::service::logs::{LogChannel, LogLine};

#[derive(Deserialize)]
pub struct TokenQuery {
    token: Option<String>,
}

fn log_line_to_event(line: LogLine) -> Result<Event, Infallible> {
    let data = serde_json::to_string(&line).unwrap_or_default();
    Ok(Event::default().data(data))
}

fn build_sse_stream(
    snapshot: Vec<LogLine>,
    receiver: tokio::sync::broadcast::Receiver<LogLine>,
) -> impl Stream<Item = Result<Event, Infallible>> {
    let snapshot_stream = stream::iter(
        snapshot
            .into_iter()
            .map(|line| Ok(log_line_to_event(line).unwrap())),
    );

    let live_stream = BroadcastStream::new(receiver).filter_map(|result| async move {
        match result {
            Ok(line) => Some(Ok(log_line_to_event(line).unwrap())),
            Err(_) => None,
        }
    });

    snapshot_stream.chain(live_stream)
}

pub async fn stream_daemon(
    State(state): State<Arc<AppState>>,
    Path(component): Path<String>,
    Query(query): Query<TokenQuery>,
    user_id_result: Result<UserId, AppError>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    authenticate(user_id_result, query.token)?;

    let channel = LogChannel::from_str(&component).map_err(|_| AppError::NotFound)?;

    let snapshot = state.services.logs().snapshot(channel.clone());
    let receiver = state.services.logs().subscribe(channel);

    Ok(Sse::new(build_sse_stream(snapshot, receiver)).keep_alive(KeepAlive::default()))
}

pub async fn stream_endpoint(
    State(state): State<Arc<AppState>>,
    Path((organization_id, project_id, branch_id)): Path<(Uuid, Uuid, Uuid)>,
    Query(query): Query<TokenQuery>,
    user_id_result: Result<UserId, AppError>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let user_id = authenticate(user_id_result, query.token)?;

    state
        .services
        .membership()
        .verify_membership(user_id, organization_id)
        .await?;

    let channel = LogChannel::ComputeEndpoint(branch_id);
    let snapshot = state.services.logs().snapshot(channel.clone());
    let receiver = state.services.logs().subscribe(channel);

    Ok(Sse::new(build_sse_stream(snapshot, receiver)).keep_alive(KeepAlive::default()))
}