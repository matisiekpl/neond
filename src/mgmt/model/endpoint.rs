use diesel::prelude::*;
use uuid::Uuid;

#[derive(diesel_derive_enum::DbEnum, Debug)]
#[ExistingTypePath = "crate::mgmt::schema::schema::sql_types::EndpointState"]
pub enum EndpointState {
    Stopped,
    Running,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::mgmt::schema::schema::endpoints)]
pub struct Endpoint {
    pub branch_id: Uuid,
    pub state: EndpointState,
    pub endpoint_port: i32,
}
