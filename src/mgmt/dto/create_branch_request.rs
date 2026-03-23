use serde::Deserialize;
use uuid::Uuid;

use crate::mgmt::model::branch::PgVersion;

#[derive(Deserialize)]
pub struct CreateBranchRequest {
    pub name: String,
    pub parent_branch_id: Option<Uuid>,
    pub pg_version: Option<PgVersion>,
}
