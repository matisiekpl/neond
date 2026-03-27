use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddMemberByEmailRequest {
    pub email: String,
}
