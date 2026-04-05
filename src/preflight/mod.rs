use std::path::PathBuf;

mod network;

const STORAGE_BROKER_PORT: u16 = 50051;
const STORAGE_CONTROLLER_PORT: u16 = 1234;
const SAFEKEEPER_PG_PORT: u16 = 5454;
const SAFEKEEPER_HTTP_PORT: u16 = 7676;
const PAGESERVER_PG_PORT: u16 = 64000;
const PAGESERVER_HTTP_PORT: u16 = 9898;
const STORAGE_CONTROLLER_DB_PORT: u16 = 5431;
const DAEMON_MGMT_DB_PORT: u16 = 5430;
const TRACER_PORT: u16 = 4318;

const MINIMUM_FREE_SPACE_GB: u64 = 3;

pub fn check(
    daemon_directory: PathBuf,
    binaries_directory: PathBuf,
    pg_proxy_port: u16,
) -> Result<(), PreflightError> {
    if !network::is_port_open(STORAGE_BROKER_PORT) {
        return Err(PreflightError::PortAlreadyReserved(STORAGE_BROKER_PORT));
    }
    if !network::is_port_open(STORAGE_CONTROLLER_PORT) {
        return Err(PreflightError::PortAlreadyReserved(STORAGE_CONTROLLER_PORT));
    }
    if !network::is_port_open(SAFEKEEPER_PG_PORT) {
        return Err(PreflightError::PortAlreadyReserved(SAFEKEEPER_PG_PORT));
    }
    if !network::is_port_open(SAFEKEEPER_HTTP_PORT) {
        return Err(PreflightError::PortAlreadyReserved(SAFEKEEPER_HTTP_PORT));
    }
    if !network::is_port_open(PAGESERVER_PG_PORT) {
        return Err(PreflightError::PortAlreadyReserved(PAGESERVER_PG_PORT));
    }
    if !network::is_port_open(PAGESERVER_HTTP_PORT) {
        return Err(PreflightError::PortAlreadyReserved(PAGESERVER_HTTP_PORT));
    }
    if !network::is_port_open(STORAGE_CONTROLLER_DB_PORT) {
        return Err(PreflightError::PortAlreadyReserved(
            STORAGE_CONTROLLER_DB_PORT,
        ));
    }
    if !network::is_port_open(DAEMON_MGMT_DB_PORT) {
        return Err(PreflightError::PortAlreadyReserved(DAEMON_MGMT_DB_PORT));
    }
    if !network::is_port_open(TRACER_PORT) {
        return Err(PreflightError::PortAlreadyReserved(TRACER_PORT));
    }
    if !network::is_port_open(pg_proxy_port) {
        return Err(PreflightError::PortAlreadyReserved(pg_proxy_port));
    }

    if !daemon_directory.exists() {
        std::fs::create_dir_all(&daemon_directory)
            .map_err(|_| PreflightError::DaemonDirectoryInitializedFailed)?;
    }

    if !binaries_directory.exists() {
        std::fs::create_dir_all(&binaries_directory)
            .map_err(|_| PreflightError::DaemonDirectoryInitializedFailed)?;
    }

    let daemon_directory_stats = fs2::statvfs(&daemon_directory)
        .map_err(|_| PreflightError::DaemonDirectoryInitializedFailed)?;
    let binaries_directory_stats = fs2::statvfs(&binaries_directory)
        .map_err(|_| PreflightError::DaemonDirectoryInitializedFailed)?;

    let available_space =
        daemon_directory_stats.available_space() + binaries_directory_stats.available_space();
    if available_space < MINIMUM_FREE_SPACE_GB * 1_000_000_000 {
        return Err(PreflightError::NotEnoughSpace);
    }

    tracing::info!(
        "Preflight check completed successfully. Available {} MBs.",
        available_space / 1_000_000
    );

    Ok(())
}

#[derive(Debug)]
pub enum PreflightError {
    PortAlreadyReserved(u16),
    NotEnoughSpace,
    DaemonDirectoryInitializedFailed,
}

impl std::fmt::Display for PreflightError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PreflightError::PortAlreadyReserved(port) => {
                write!(f, "Port {} is already reserved", port)
            }
            PreflightError::NotEnoughSpace => write!(
                f,
                "Not enough disk space. Required available space: {} GB",
                MINIMUM_FREE_SPACE_GB
            ),
            PreflightError::DaemonDirectoryInitializedFailed => {
                write!(f, "Failed to initialize daemon directory")
            }
        }
    }
}

impl std::error::Error for PreflightError {}
