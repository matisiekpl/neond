use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
}
