use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateProjectRequest {
    pub name: String,
}
