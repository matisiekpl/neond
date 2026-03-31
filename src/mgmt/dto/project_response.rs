use chrono::NaiveDateTime;
use serde::Serialize;
use uuid::Uuid;
use crate::mgmt::model::project::PgVersion;

#[derive(Serialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub pg_version: PgVersion,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gc_period: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gc_horizon: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitr_interval: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_distance: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_timeout: Option<String>,
}
