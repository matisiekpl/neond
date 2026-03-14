use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct OrganizationResponse {
    pub id: Uuid,
    pub name: String,
}
