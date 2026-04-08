use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
pub struct CheckpointStatus {
    pub all_in_sync: bool,
    pub max_checkpoint_timeout: Option<Duration>,
}