use serde::Deserialize;

use crate::mgmt::model::project::PgVersion;

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub pg_version: Option<PgVersion>,
}
