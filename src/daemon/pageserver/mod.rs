use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize)]
struct PageserverConfig {
    availability_zone: String,
    pg_distrib_dir: String,
    broker_endpoint: String,
    listen_pg_addr: String,
    listen_http_addr: String,
    remote_storage: PageserverRemoteStorage,
    control_plane_api: String,
}

#[derive(Serialize)]
struct PageserverRemoteStorage {
    local_path: String,
}

#[derive(Serialize, Deserialize)]
struct PageserverMetadata {
    pub host: String,
    pub http_host: String,
    pub http_port: i64,
    pub port: i64,
}

pub fn write_pageserver_init_files(daemon_directory: &PathBuf) -> Result<(), anyhow::Error> {
    let config = PageserverConfig {
        availability_zone: "neond-1".to_string(),
        pg_distrib_dir: daemon_directory
            .join("binaries/pg_install")
            .to_str()
            .unwrap()
            .to_string(),
        broker_endpoint: "http://127.0.0.1:50051/".to_string(),
        listen_pg_addr: "127.0.0.1:64000".to_string(),
        listen_http_addr: "127.0.0.1:9898".to_string(),
        remote_storage: PageserverRemoteStorage {
            // TODO(matisiekpl): add support for remote storage
            local_path: daemon_directory
                .join("pageserver_1")
                .to_str()
                .unwrap()
                .to_string(),
        },
        control_plane_api: "http://127.0.0.1:1234/upcall/v1/".to_string(),
    };

    let identity_filename = daemon_directory.join("pageserver/identity.toml");
    fs::write(identity_filename.clone(), "id=1")?;
    tracing::info!(
        "Wrote pageserver identity.toml to {}",
        identity_filename.display()
    );

    let config_filename = daemon_directory.join("pageserver/pageserver.toml");
    let toml = toml::to_string_pretty(&config)?;
    fs::write(config_filename.clone(), toml)?;
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
    fs::write(
        metadata_filename.clone(),
        serde_json::to_string_pretty(&metadata)?,
    )?;
    tracing::info!(
        "Wrote pageserver metadata.json to {}",
        metadata_filename.display()
    );

    Ok(())
}
