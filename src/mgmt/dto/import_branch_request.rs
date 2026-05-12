use serde::Deserialize;

#[derive(Deserialize)]
pub struct ImportBranchRequest {
    pub name: String,
    pub source_connection_string: String,
}
