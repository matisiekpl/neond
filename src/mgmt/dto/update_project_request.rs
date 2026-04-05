use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub gc_period: Option<String>,
    pub gc_horizon: Option<u64>,
    pub pitr_interval: Option<String>,
    pub checkpoint_distance: Option<u64>,
    pub checkpoint_timeout: Option<String>,
}
