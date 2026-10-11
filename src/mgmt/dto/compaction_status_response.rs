use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CompactionStatusResponse {
    pub running: bool,
    pub queued_jobs: usize,
}
