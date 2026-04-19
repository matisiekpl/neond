use serde::Deserialize;

#[derive(Deserialize)]
pub struct RestoreBranchRequest {
    pub lsn: String,
}