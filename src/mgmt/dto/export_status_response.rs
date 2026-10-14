use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::mgmt::dto::export_branch_request::ExportFormat;
use crate::mgmt::model::project::PgVersion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportState {
    #[default]
    Idle,
    Preparing,
    Dumping,
    Finalizing,
    Completed,
    Failed,
    Cancelled,
}

impl ExportState {
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            ExportState::Preparing | ExportState::Dumping | ExportState::Finalizing
        )
    }
}

#[derive(Serialize, Clone, Default)]
pub struct ExportStatusResponse {
    pub export_id: Option<Uuid>,
    pub state: ExportState,
    pub format: Option<ExportFormat>,
    pub database: Option<String>,
    pub pg_version: Option<PgVersion>,
    pub pg_dump_version: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub progress: f64,
    pub current_table: Option<String>,
    pub tables_total: u64,
    pub tables_done: u64,
    pub bytes_total_estimate: u64,
    pub bytes_written: u64,
    pub file_name: Option<String>,
}
