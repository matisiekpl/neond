use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use axum::extract::State;
use axum::http::{HeaderValue, StatusCode, header::CONTENT_TYPE};
use axum::response::IntoResponse;
use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

use crate::mgmt::dto::error::AppError;
use crate::mgmt::handler::AppState;
use crate::utils::metrics::sanitize_metric_slug;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct BranchLabelSet {
    branch: String,
    project: String,
}

pub async fn scrape(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let snapshot = state.services.metric().snapshot_latest().await;

    let mut registry = Registry::default();
    let mut families: HashMap<String, Family<BranchLabelSet, Gauge<f64, AtomicU64>>> = HashMap::new();

    for (key, value) in &snapshot.samples {
        let metric_name = sanitize_metric_slug(&key.slug);

        let family = families.entry(metric_name.clone()).or_insert_with(|| {
            let family: Family<BranchLabelSet, Gauge<f64, AtomicU64>> = Family::default();
            registry.register(metric_name.clone(), metric_name.clone(), family.clone());
            family
        });

        let label_set = match key.branch_id {
            Some(branch_id) => match snapshot.branch_labels.get(&branch_id) {
                Some(labels) => BranchLabelSet {
                    branch: labels.branch_name.clone(),
                    project: labels.project_name.clone(),
                },
                None => continue,
            },
            None => BranchLabelSet {
                branch: String::new(),
                project: String::new(),
            },
        };

        family.get_or_create(&label_set).set(*value);
    }

    let mut buffer = String::new();
    encode(&mut buffer, &registry).map_err(|error| AppError::Internal(error.to_string()))?;

    Ok((
        StatusCode::OK,
        [(CONTENT_TYPE, HeaderValue::from_static("application/openmetrics-text; version=1.0.0; charset=utf-8"))],
        buffer,
    ))
}