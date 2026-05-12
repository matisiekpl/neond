use std::env::current_dir;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::auth::DaemonAuth;
use crate::mgmt::dto::error::{AppError, Result};

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
    pub(crate) pgbouncer_bin: Option<PathBuf>,
    pub(crate) remote_storage_config: Option<RemoteStorageConfig>,
    pub(crate) port_range: PortRange,
    pub(crate) hostname: Option<String>,
    pub(crate) pg_proxy_port: u16,
    pub(crate) component_auth: Arc<DaemonAuth>,
    pub(crate) backup_interval: Duration,
}

fn discover_pgbouncer() -> Option<PathBuf> {
    if let Ok(value) = std::env::var("PGBOUNCER_BIN") {
        let path = PathBuf::from(value);
        if path.exists() {
            return Some(path);
        }
        tracing::warn!("PGBOUNCER_BIN set to {} but file does not exist", path.display());
    }
    for candidate in [
        "/usr/bin/pgbouncer",
        "/usr/sbin/pgbouncer",
        "/usr/local/bin/pgbouncer",
        "/opt/homebrew/bin/pgbouncer",
    ] {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

impl Config {
    pub fn new() -> Result<Self> {
        let port: u16 = match std::env::var("PORT") {
            Ok(port) => {
                port.parse()
                    .map_err(|error: std::num::ParseIntError| AppError::ApplicationStartupFailed {
                        reason: format!("PORT is invalid: {}", error),
                    })?
            }
            Err(_) => 3000,
        };
        let server_secret =
            std::env::var("SERVER_SECRET").map_err(|_| AppError::ServerSecretNotConfigured)?;
        let daemon_directory = current_dir()
            .map_err(|error| AppError::WorkingDirectoryInvalid {
                path: error.to_string(),
            })?
            .join("neon_daemon_data");

        let build_profile = "release";
        let neon_binaries_directory = match std::env::var("NEON_BINARIES_DIR") {
            Ok(value) => PathBuf::from(value),
            Err(_) => current_dir()
                .map_err(|error| AppError::WorkingDirectoryInvalid {
                    path: error.to_string(),
                })?
                .join("neon")
                .join("target")
                .join(build_profile),
        };
        let pg_install_directory = match std::env::var("PG_INSTALL_DIR") {
            Ok(value) => PathBuf::from(value),
            Err(_) => current_dir()
                .map_err(|error| AppError::WorkingDirectoryInvalid {
                    path: error.to_string(),
                })?
                .join("neon")
                .join("pg_install"),
        };

        let pgbouncer_bin = discover_pgbouncer();
        match &pgbouncer_bin {
            Some(path) => tracing::info!("pgbouncer found at {}, pooling enabled", path.display()),
            None => tracing::warn!("pgbouncer binary not found, pooling disabled"),
        }

        let remote_storage_config = if std::env::var("AWS_S3_BUCKET").is_ok()
            && std::env::var("AWS_REGION").is_ok()
            && std::env::var("AWS_ACCESS_KEY_ID").is_ok()
            && std::env::var("AWS_SECRET_ACCESS_KEY").is_ok()
        {
            let bucket = std::env::var("AWS_S3_BUCKET").map_err(|error| {
                AppError::ApplicationStartupFailed {
                    reason: format!("AWS_S3_BUCKET: {}", error),
                }
            })?;
            let region = std::env::var("AWS_REGION").map_err(|error| {
                AppError::ApplicationStartupFailed {
                    reason: format!("AWS_REGION: {}", error),
                }
            })?;
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
                let start_text =
                    parts
                        .next()
                        .ok_or_else(|| AppError::PortRangeMisconfigured {
                            value: port_range.clone(),
                        })?;
                let end_text =
                    parts
                        .next()
                        .ok_or_else(|| AppError::PortRangeMisconfigured {
                            value: port_range.clone(),
                        })?;
                let start =
                    start_text
                        .parse()
                        .map_err(|_| AppError::PortRangeMisconfigured {
                            value: port_range.clone(),
                        })?;
                let end = end_text
                    .parse()
                    .map_err(|_| AppError::PortRangeMisconfigured {
                        value: port_range.clone(),
                    })?;
                PortRange(start, end)
            }
            Err(_) => PortRange(49152, 65535),
        };

        let pg_proxy_port = match std::env::var("PG_PROXY_PORT") {
            Ok(port) => {
                port.parse()
                    .map_err(|error: std::num::ParseIntError| AppError::ApplicationStartupFailed {
                        reason: format!("PG_PROXY_PORT is invalid: {}", error),
                    })?
            }
            Err(_) => 5432,
        };

        let backup_interval = match std::env::var("BACKUP_INTERVAL") {
            Ok(value) => humantime::parse_duration(&value).map_err(|error| {
                AppError::ApplicationStartupFailed {
                    reason: format!("BACKUP_INTERVAL is invalid: {}", error),
                }
            })?,
            Err(_) => Duration::from_secs(30 * 60),
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
            pgbouncer_bin,
            remote_storage_config,
            port_range,
            hostname,
            pg_proxy_port,
            component_auth: Arc::new(DaemonAuth::generate()?),
            backup_interval,
        })
    }
}
