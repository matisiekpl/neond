use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::mgmt::schema::schema::compute_metric_samples)]
pub struct ComputeMetricSample {
    pub id: Uuid,
    pub branch_id: Option<Uuid>,
    pub recorded_at: NaiveDateTime,
    pub slug: String,
    pub value: f64,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::mgmt::schema::schema::compute_metric_samples)]
pub struct NewComputeMetricSample {
    pub id: Uuid,
    pub branch_id: Option<Uuid>,
    pub recorded_at: NaiveDateTime,
    pub slug: String,
    pub value: f64,
}
