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
}
