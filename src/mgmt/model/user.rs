use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::mgmt::schema::schema::users)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
