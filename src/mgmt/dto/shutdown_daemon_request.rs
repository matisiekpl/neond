use serde::Deserialize;

#[derive(Deserialize)]
pub struct ShutdownDaemonRequest {
    pub wait_for_checkpoints: bool,
}