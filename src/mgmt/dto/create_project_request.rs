use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
}
