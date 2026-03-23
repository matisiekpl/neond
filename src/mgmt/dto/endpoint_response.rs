use serde::Serialize;
use uuid::Uuid;

use crate::mgmt::compute::ComputeEndpointStatus;

#[derive(Serialize)]
pub struct EndpointResponse {
    pub branch_id: Uuid,
    pub status: ComputeEndpointStatus,
    pub port: u16,
}
