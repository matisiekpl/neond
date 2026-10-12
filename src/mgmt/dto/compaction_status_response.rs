use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CompactionStatusResponse {
    pub running: bool,
    /// When the last compaction run of this branch started and finished, if any since neond started.
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    /// The pageserver's error if the last run failed.
    pub error: Option<String>,
    /// Physical size of the branch's timeline before and after the last run.
    pub size_before: Option<u64>,
    pub size_after: Option<u64>,
}
