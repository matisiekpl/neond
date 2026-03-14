use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub organization_id: Uuid,
    pub name: String,
}
