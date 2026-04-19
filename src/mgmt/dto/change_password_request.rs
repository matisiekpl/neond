use serde::Deserialize;

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub password: String,
}