use serde::Serialize;

#[derive(Serialize)]
pub struct LoginUserResponse {
    pub token: String,
}
