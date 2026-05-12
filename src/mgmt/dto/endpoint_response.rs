use serde::Serialize;
use uuid::Uuid;

use crate::mgmt::compute::ComputeEndpointStatus;

#[derive(Serialize)]
pub struct EndpointResponse {
    pub branch_id: Uuid,
    pub status: ComputeEndpointStatus,
    pub port: u16,
    pub pooler_port: Option<u16>,
    pub sni_hostname: Option<String>,
    pub pooler_sni_hostname: Option<String>,
    pub password: String,
}
