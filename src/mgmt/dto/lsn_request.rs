use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LsnRequest {
    pub timestamp: DateTime<Utc>,
}