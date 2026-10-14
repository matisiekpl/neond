use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct ExportDownloadTokenResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}
