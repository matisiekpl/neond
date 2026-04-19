use serde::Serialize;

#[derive(Serialize)]
pub struct SetupResponse {
    pub registration_open: bool,
}
