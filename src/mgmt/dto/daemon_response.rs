use serde::Serialize;

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
    pub organization_name: String,
    pub project_name: String,
    pub branch_name: String,
    pub port: u16,
    pub sni: Option<String>,
}

#[derive(Serialize)]
pub struct DaemonResponse {
    pub storage: StorageInfo,
    pub mappings: Vec<MappingInfo>,
}