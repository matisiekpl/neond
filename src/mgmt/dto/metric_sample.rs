use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Serialize)]
pub struct MetricSample {
    pub recorded_at: NaiveDateTime,
    pub slug: String,
    pub value: f64,
}
