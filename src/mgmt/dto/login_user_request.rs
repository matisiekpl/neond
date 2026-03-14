use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginUserRequest {
    pub email: String,
    pub password: String,
}
