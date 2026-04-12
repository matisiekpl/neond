use std::env::current_dir;
use std::path::PathBuf;
use std::sync::Arc;

use crate::auth::DaemonAuth;

#[derive(Clone)]
pub struct RemoteStorageConfig {
    pub(crate) bucket: String,
    pub(crate) region: String,
}

#[derive(Clone)]
pub struct PortRange(pub u16, pub u16);

#[derive(Clone)]
pub struct Config {
    pub(crate) port: u16,
    pub(crate) server_secret: String,
    pub(crate) daemon_directory: PathBuf,
    pub(crate) neon_binaries_directory: PathBuf,
    pub(crate) pg_install_directory: PathBuf,
    pub(crate) remote_storage_config: Option<RemoteStorageConfig>,
    pub(crate) port_range: PortRange,
    pub(crate) hostname: Option<String>,
    pub(crate) pg_proxy_port: u16,
    pub(crate) component_auth: Arc<DaemonAuth>,
}

impl Config {
    pub fn new() -> Result<Self, anyhow::Error> {
        let port: u16 = match std::env::var("PORT") {
            Ok(port) => port.parse()?,
            Err(_) => 3000,
        };
        let server_secret =
            std::env::var("SERVER_SECRET").map_err(|_| anyhow::anyhow!("SERVER_SECRET not set"))?;
        let daemon_directory = current_dir()?.join("neon_daemon_data");

        let build_profile = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        let neon_binaries_directory = match std::env::var("NEON_BINARIES_DIR") {
            Ok(value) => PathBuf::from(value),
            Err(_) => current_dir()?
                .join("neon")
                .join("target")
                .join(build_profile),
        };
        let pg_install_directory = match std::env::var("PG_INSTALL_DIR") {
            Ok(value) => PathBuf::from(value),
            Err(_) => current_dir()?.join("neon").join("pg_install"),
        };

        let remote_storage_config = if std::env::var("AWS_S3_BUCKET").is_ok()
            && std::env::var("AWS_REGION").is_ok()
            && std::env::var("AWS_ACCESS_KEY_ID").is_ok()
            && std::env::var("AWS_SECRET_ACCESS_KEY").is_ok()
        {
            let bucket = std::env::var("AWS_S3_BUCKET")?;
            let region = std::env::var("AWS_REGION")?;
            tracing::info!(
                "Using remote storage - bucket: {}, region: {}",
                bucket,
                region
            );
            Some(RemoteStorageConfig { bucket, region })
        } else {
            tracing::info!("Using local storage");
            None
        };
        let port_range = match std::env::var("PORT_RANGE") {
            Ok(port_range) => {
                let mut parts = port_range.split('-');
                let start = parts.next().unwrap().parse()?;
                let end = parts.next().unwrap().parse()?;
                PortRange(start, end)
            }
            Err(_) => PortRange(49152, 65535),
        };

        let pg_proxy_port = match std::env::var("PG_PROXY_PORT") {
            Ok(port) => port.parse()?,
            Err(_) => 5432,
        };

        let hostname = std::env::var("PG_HOSTNAME").ok();
        if let Some(hostname) = hostname.clone() {
            tracing::info!("Using hostname *.{}:{}", hostname, pg_proxy_port);
        } else {
            tracing::info!("Hostname (PG_HOSTNAME) is not set, TLS SNI routing disabled");
        }

        Ok(Self {
            port,
            server_secret,
            daemon_directory,
            neon_binaries_directory,
            pg_install_directory,
            remote_storage_config,
            port_range,
            hostname,
            pg_proxy_port,
            component_auth: Arc::new(DaemonAuth::generate()?),
        })
    }
}
