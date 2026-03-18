use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateBranchRequest {
    pub name: String,
}
