use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
}
