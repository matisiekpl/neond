use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::mgmt::schema::schema::branches)]
pub struct Branch {
    pub id: Uuid,
    pub name: String,
    pub parent_branch_id: Option<Uuid>,
    pub timeline_id: Uuid,
    pub project_id: Uuid,
    pub password: String,
    pub slug: String,
}
