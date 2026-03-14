use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateOrganizationRequest {
    pub name: String,
}
