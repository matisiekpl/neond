use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LsnResponse {
    pub lsn: String,
    pub kind: String,
}