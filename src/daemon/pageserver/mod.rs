use crate::auth::DaemonAuth;
use crate::mgmt::dto::config::RemoteStorageConfig;
use crate::mgmt::dto::error::{AppError, Result};
use neon_utils::auth::Scope;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct PageserverConfig {
    availability_zone: String,
    pg_distrib_dir: String,
    broker_endpoint: String,
    listen_pg_addr: String,
    listen_http_addr: String,
    remote_storage: PageserverRemoteStorage,
    control_plane_api: String,
    auth_validation_public_key_path: String,
    http_auth_type: String,
    pg_auth_type: String,
    control_plane_api_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum PageserverRemoteStorage {
    LocalPath {
        local_path: String,
    },
    AmazonBucket {
        bucket_name: String,
        bucket_region: String,
        prefix_in_bucket: String,
    },
}

#[derive(Serialize, Deserialize)]
struct PageserverMetadata {
    pub host: String,
    pub http_host: String,
    pub http_port: i64,
    pub port: i64,
}

fn path_to_string(path: &Path) -> Result<String> {
    path.to_str()
        .map(|value| value.to_string())
        .ok_or_else(|| AppError::PageserverConfigWriteFailed {
            reason: format!("path contains non-UTF-8 characters: {}", path.display()),
        })
}

pub fn write_pageserver_init_files(
    daemon_directory: &PathBuf,
    pg_install_directory: &PathBuf,
    remote_storage_config: &Option<RemoteStorageConfig>,
    component_auth: &DaemonAuth,
) -> Result<()> {
    let config = PageserverConfig {
        availability_zone: "neond-1".to_string(),
        pg_distrib_dir: path_to_string(pg_install_directory)?,
        broker_endpoint: "http://127.0.0.1:50051/".to_string(),
        listen_pg_addr: "127.0.0.1:64000".to_string(),
        listen_http_addr: "127.0.0.1:9898".to_string(),
        remote_storage: match remote_storage_config {
            None => PageserverRemoteStorage::LocalPath {
                local_path: path_to_string(&daemon_directory.join("pageserver_1"))?,
            },
            Some(amazon_bucket_config) => PageserverRemoteStorage::AmazonBucket {
                bucket_name: amazon_bucket_config.bucket.clone(),
                bucket_region: amazon_bucket_config.region.clone(),
                prefix_in_bucket: "layers".to_string(),
            },
        },
        control_plane_api: "http://127.0.0.1:1234/upcall/v1/".to_string(),
        auth_validation_public_key_path: path_to_string(component_auth.public_key_path())?,
        http_auth_type: "NeonJWT".to_string(),
        pg_auth_type: "NeonJWT".to_string(),
        control_plane_api_token: component_auth.generate_token(Scope::GenerationsApi, None)?,
    };

    let identity_filename = daemon_directory.join("pageserver/identity.toml");
    fs::write(identity_filename.clone(), "id=1").map_err(|error| {
        AppError::PageserverConfigWriteFailed {
            reason: error.to_string(),
        }
    })?;
    tracing::info!(
        "Wrote pageserver identity.toml to {}",
        identity_filename.display()
    );

    let config_filename = daemon_directory.join("pageserver/pageserver.toml");
    let toml =
        toml::to_string_pretty(&config).map_err(|error| AppError::PageserverConfigWriteFailed {
            reason: error.to_string(),
        })?;
    fs::write(config_filename.clone(), toml).map_err(|error| {
        AppError::PageserverConfigWriteFailed {
            reason: error.to_string(),
        }
    })?;
    tracing::info!(
        "Wrote pageserver config.toml to {}",
        config_filename.display()
    );

    let metadata_filename = daemon_directory.join("pageserver/metadata.json");
    let metadata = PageserverMetadata {
        host: "127.0.0.1".to_string(),
        http_host: "127.0.0.1".to_string(),
        http_port: 9898,
        port: 64000,
    };
    let metadata_json = serde_json::to_string_pretty(&metadata).map_err(|error| {
        AppError::PageserverConfigWriteFailed {
            reason: error.to_string(),
        }
    })?;
    fs::write(metadata_filename.clone(), metadata_json).map_err(|error| {
        AppError::PageserverConfigWriteFailed {
            reason: error.to_string(),
        }
    })?;
    tracing::info!(
        "Wrote pageserver metadata.json to {}",
        metadata_filename.display()
    );

    Ok(())
}
