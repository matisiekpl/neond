use neon_utils::lsn::Lsn;
use serde::Serialize;
use uuid::Uuid;

use crate::mgmt::compute::ComputeEndpointStatus;

#[derive(Serialize)]
pub struct BranchResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub parent_branch_id: Option<Uuid>,
    pub timeline_id: Uuid,
    pub endpoint_status: ComputeEndpointStatus,
    pub remote_consistent_lsn_visible: Lsn,
    pub last_record_lsn: Lsn,
}
