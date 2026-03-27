use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(diesel_derive_enum::DbEnum, Debug, Clone, Serialize, Deserialize)]
#[ExistingTypePath = "crate::mgmt::schema::schema::sql_types::PgVersion"]
pub enum PgVersion {
    #[db_rename = "v14"]
    V14,
    #[db_rename = "v15"]
    V15,
    #[db_rename = "v16"]
    V16,
    #[db_rename = "v17"]
    V17,
}

impl fmt::Display for PgVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PgVersion::V14 => "v14",
            PgVersion::V15 => "v15",
            PgVersion::V16 => "v16",
            PgVersion::V17 => "v17",
        };
        write!(f, "{}", s)
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::mgmt::schema::schema::projects)]
pub struct Project {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub pg_version: PgVersion,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
