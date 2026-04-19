use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub name: String,
    pub email: String,
    pub is_admin: bool,
}
