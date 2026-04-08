use neon_utils::lsn::Lsn;
use serde::Serialize;
use uuid::Uuid;

use crate::mgmt::compute::ComputeEndpointStatus;

#[derive(Serialize)]
pub struct LocalStorageInfo {
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub used_percent: f64,
}

#[derive(Serialize)]
pub struct RemoteStorageInfo {
    pub bucket: String,
    pub region: String,
    pub aws_access_key_id: String,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum StorageInfo {
    Local(LocalStorageInfo),
    Remote(RemoteStorageInfo),
}

#[derive(Serialize)]
pub struct MappingInfo {
    pub branch_id: Uuid,
    pub organization_id: Uuid,
    pub organization_name: String,
    pub project_id: Uuid,
    pub project_name: String,
    pub branch_name: String,
    pub slug: String,
    pub endpoint_status: ComputeEndpointStatus,
    pub port: Option<u16>,
    pub sni: Option<String>,
    pub last_record_lsn: Lsn,
    pub remote_consistent_lsn_visible: Lsn,
    pub current_logical_size: u64,
}

#[derive(Serialize)]
pub struct DaemonResponse {
    pub hostname: Option<String>,
    pub build_version: String,
    pub storage: StorageInfo,
    pub mappings: Vec<MappingInfo>,
}