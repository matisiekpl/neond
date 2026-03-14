use serde::Serialize;

#[derive(Serialize)]
pub struct RegisterUserResponse {
    pub token: String,
}
