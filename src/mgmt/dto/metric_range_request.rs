use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MetricRangeRequest {
    pub from: Option<NaiveDateTime>,
    pub to: Option<NaiveDateTime>,
}
