use std::path::{Path, PathBuf};

use super::PreflightError;

const NEON_COMPONENT_BINARIES: &[&str] = &[
    "safekeeper",
    "pageserver",
    "compute_ctl",
    "storage_broker",
    "storage_controller",
];

const PG_INSTALL_REQUIRED_PATHS: &[&str] = &["v17/bin/postgres", "v17/bin/initdb", "v17/lib"];

pub fn check_neon_binaries(neon_binaries_directory: &Path) -> Result<(), PreflightError> {
    if !neon_binaries_directory.exists() {
        return Err(PreflightError::MissingBinary(
            neon_binaries_directory.to_path_buf(),
        ));
    }
    for binary_name in NEON_COMPONENT_BINARIES {
        let binary_path = neon_binaries_directory.join(binary_name);
        if !binary_path.exists() {
            return Err(PreflightError::MissingBinary(binary_path));
        }
    }
    Ok(())
}

pub fn check_pg_install(pg_install_directory: &Path) -> Result<(), PreflightError> {
    if !pg_install_directory.exists() {
        return Err(PreflightError::MissingPgInstall(
            pg_install_directory.to_path_buf(),
        ));
    }
    for required in PG_INSTALL_REQUIRED_PATHS {
        let required_path: PathBuf = pg_install_directory.join(required);
        if !required_path.exists() {
            return Err(PreflightError::MissingPgInstall(required_path));
        }
    }
    Ok(())
}