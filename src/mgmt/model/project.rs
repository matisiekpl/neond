use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::mgmt::schema::schema::projects)]
pub struct Project {
    pub id: Uuid,
    pub organization_id: Uuid,
}
