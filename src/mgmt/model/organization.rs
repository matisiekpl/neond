use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::organizations)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
}
