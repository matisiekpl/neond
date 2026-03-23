use serde::Serialize;
use uuid::Uuid;

use crate::mgmt::model::branch::PgVersion;

#[derive(Serialize)]
pub struct BranchResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub parent_branch_id: Option<Uuid>,
    pub timeline_id: Uuid,
    pub pg_version: PgVersion,
}
