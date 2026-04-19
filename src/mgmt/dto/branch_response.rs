use chrono::NaiveDateTime;
use neon_utils::lsn::Lsn;
use serde::Serialize;
use uuid::Uuid;

use crate::mgmt::compute::ComputeEndpointStatus;
use crate::mgmt::dto::config::Config;

#[derive(Serialize)]
pub struct BranchResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub slug: String,
    pub parent_branch_id: Option<Uuid>,
    pub timeline_id: Uuid,
    pub ancestor_timeline_id: Option<Uuid>,
    pub ancestor_lsn: Option<Lsn>,
    pub endpoint_status: ComputeEndpointStatus,
    pub remote_consistent_lsn_visible: Lsn,
    pub last_record_lsn: Lsn,
    pub current_logical_size: u64,
    pub connection_string: Option<String>,
    pub password: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

