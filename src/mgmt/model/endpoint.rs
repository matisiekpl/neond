use diesel::prelude::*;
use uuid::Uuid;

#[derive(diesel_derive_enum::DbEnum, Debug)]
pub enum EndpointState {
    Stopped,
    Running,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::endpoints)]
pub struct Endpoint {
    pub branch_id: Uuid,
    pub state: EndpointState,
    pub endpoint_port: i32,
}
