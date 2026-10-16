use serde::Serialize;

use crate::mgmt::compute::settings::{ComputeSettings, ComputeSettingsOverrides};

#[derive(Serialize)]
pub struct ComputeSettingsResponse {
    pub shared_buffers_mb: i32,
    pub overrides: ComputeSettingsOverrides,
    pub effective: ComputeSettings,
    pub max_shared_buffers_mb: i32,
    pub memory: ComputeMemoryOverview,
}

#[derive(Serialize)]
pub struct ComputeMemoryOverview {
    pub host_total_mb: i64,
    pub host_available_mb: i64,
    pub shared_buffers_budget_mb: i64,
    pub other_running_computes: usize,
    pub other_running_shared_buffers_mb: i64,
    pub recommended_shared_buffers_mb: i32,
}
