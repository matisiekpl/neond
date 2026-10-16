use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateComputeSettingsRequest {
    pub shared_buffers_mb: i32,
    pub effective_cache_size_mb: Option<i32>,
    pub maintenance_work_mem_mb: Option<i32>,
    pub work_mem_mb: Option<i32>,
    pub max_connections: Option<i32>,
}
