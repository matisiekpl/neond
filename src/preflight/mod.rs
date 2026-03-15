
mod network;

const STORAGE_BROKER_PORT: u16 = 50051;
const STORAGE_CONTROLLER_PORT: u16 = 1234;
const SAFEKEEPER_PG_PORT: u16 = 5454;
const SAFEKEEPER_HTTP_PORT: u16 = 7676;
const PAGESERVER_PG_PORT: u16 = 64000;
const PAGESERVER_HTTP_PORT: u16 = 9898;

const MINIMUM_FREE_SPACE_GB: u64 = 3;

pub fn check(daemon_directory: std::path::PathBuf) -> Result<(), PreflightError> {
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

    if !daemon_directory.exists() {
        std::fs::create_dir_all(&daemon_directory)
            .map_err(|_| PreflightError::DaemonDirectoryInitializedFailed)?;
    }

    let daemon_directory_stats = fs2::statvfs(&daemon_directory)
        .map_err(|_| PreflightError::DaemonDirectoryInitializedFailed)?;

    if daemon_directory_stats.available_space() < MINIMUM_FREE_SPACE_GB * 1_000_000_000 {
        return Err(PreflightError::NotEnoughSpace);
    }

    tracing::info!(
        "Preflight check completed successfully. Available {} MBs.",
        daemon_directory_stats.available_space() / 1_000_000
    );

    Ok(())
}

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
