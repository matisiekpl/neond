use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateBranchRequest {
    pub name: String,
    pub parent_branch_id: Option<Uuid>,
}
