use std::path::PathBuf;

pub struct Unpacker {
    safekeeper_path: PathBuf,
    pageserver_path: PathBuf,
    compute_ctl_path: PathBuf,
    storage_broker_path: PathBuf,
    storage_controller_path: PathBuf,
    pg_install_path: PathBuf,
}

const SAFEKEEPER_BINARY: &[u8] = include_bytes!("../../neon/target/debug/safekeeper");
const PAGESERVER_BINARY: &[u8] = include_bytes!("../../neon/target/debug/pageserver");
const COMPUTE_CTL_BINARY: &[u8] = include_bytes!("../../neon/target/debug/compute_ctl");
const STORAGE_BROKER_BINARY: &[u8] = include_bytes!("../../neon/target/debug/storage_broker");
const STORAGE_CONTROLLER_BINARY: &[u8] =
    include_bytes!("../../neon/target/debug/storage_controller");
const PG_INSTALL_TAR: &[u8] = include_bytes!("../../neon/target/pg_install.tar");

impl Unpacker {
    pub fn new(daemon_directory: PathBuf) -> Result<Self, std::io::Error> {
        let binaries_directory = daemon_directory.join("binaries");
        std::fs::create_dir_all(&binaries_directory)?;
        Ok(Unpacker {
            compute_ctl_path: binaries_directory.join("compute_ctl"),
            storage_broker_path: binaries_directory.join("storage_broker"),
            storage_controller_path: binaries_directory.join("storage_controller"),
            safekeeper_path: binaries_directory.join("safekeeper"),
            pageserver_path: binaries_directory.join("pageserver"),
            pg_install_path: binaries_directory.join("pg_install"),
        })
    }

    pub fn unpack(self) -> Result<(), anyhow::Error> {
        if cfg!(debug_assertions) {
            tracing::info!("Debug mode enabled, skipping unpacking");
            return Ok(());
        }

        self.unpack_neon_binaries()?;
        self.unpack_pg_install()?;
        Ok(())
    }

    fn unpack_neon_binaries(&self) -> Result<(), std::io::Error> {
        tracing::info!("Unpacking Neon binaries...");
        let binaries = [
            (&self.pageserver_path, PAGESERVER_BINARY, "pageserver"),
            (&self.safekeeper_path, SAFEKEEPER_BINARY, "safekeeper"),
            (&self.compute_ctl_path, COMPUTE_CTL_BINARY, "compute_ctl"),
            (
                &self.storage_broker_path,
                STORAGE_BROKER_BINARY,
                "storage_broker",
            ),
            (
                &self.storage_controller_path,
                STORAGE_CONTROLLER_BINARY,
                "storage_controller",
            ),
        ];

        for (path, binary, name) in &binaries {
            std::fs::write(path, binary)?;
            tracing::info!("{} binary unpacked successfully", name);

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
            }
        }

        Ok(())
    }

    fn unpack_pg_install(&self) -> Result<(), std::io::Error> {
        tracing::info!("Unpacking Postgres binaries...");
        std::fs::create_dir_all(&self.pg_install_path)?;
        let mut archive = tar::Archive::new(PG_INSTALL_TAR);
        archive.unpack(&self.pg_install_path)?;
        tracing::info!("pg_install directory unpacked successfully");
        Ok(())
    }
}
