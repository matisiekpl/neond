use crate::daemon::stdout::wait_for_output;
use anyhow::anyhow;
use std::path::PathBuf;
use std::process::Stdio;
use crate::daemon::death;

pub struct Postgres {
    name: &'static str,
    initdb_binary_path: PathBuf,
    postgres_binary_path: PathBuf,
    postgres_lib_path: PathBuf,
    data_directory: PathBuf,
    port: u16,
    process: Option<std::process::Child>,
    verbose: bool,
}

impl Postgres {
    pub fn new(
        name: &'static str,
        daemon_directory: PathBuf,
        data_directory_suffix: &'static str,
        port: u16,
    ) -> Self {
        Postgres {
            name,
            initdb_binary_path: daemon_directory.join("binaries/pg_install/v17/bin/initdb"),
            postgres_binary_path: daemon_directory.join("binaries/pg_install/v17/bin/postgres"),
            postgres_lib_path: daemon_directory.join("binaries/pg_install/v17/lib"),
            data_directory: daemon_directory
                .join("daemon_data")
                .join(data_directory_suffix),
            port,
            process: None,
            verbose: cfg!(debug_assertions),
        }
    }

    pub fn init(&self) -> Result<(), anyhow::Error> {
        if self.data_directory.join("postgresql.conf").exists() {
            tracing::info!("Postgres data dir for {} already initialized", self.name);
            return Ok(());
        }

        let exit_status = std::process::Command::new(self.initdb_binary_path.clone())
            .env(
                "DYLD_LIBRARY_PATH",
                self.postgres_lib_path.to_str().unwrap(),
            )
            .env("LD_LIBRARY_PATH", self.postgres_lib_path.to_str().unwrap())
            .arg("-U")
            .arg("neon")
            .arg("-D")
            .arg(self.data_directory.to_str().unwrap())
            .stdout(Stdio::piped())
            .spawn()?
            .wait()?;
        let exit_code = exit_status
            .code()
            .ok_or(anyhow!("Failed to initialize postgres"))?;
        if exit_code != 0 {
            return Err(anyhow!("Failed to initialize postgres"));
        }
        tracing::info!("Postgres data dir for {} initialized", self.name);
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), anyhow::Error> {
        let mut cmd = std::process::Command::new(self.postgres_binary_path.clone());
        death::configure_death_signal(&mut cmd);
        let mut child = cmd
            .env(
                "DYLD_LIBRARY_PATH",
                self.postgres_lib_path.to_str().unwrap(),
            )
            .env("LD_LIBRARY_PATH", self.postgres_lib_path.to_str().unwrap())
            .arg("-D")
            .arg(self.data_directory.to_str().unwrap())
            .arg("-p")
            .arg(self.port.to_string())
            .stderr(Stdio::piped())
            .spawn()?;

        let stderr = child.stderr.take().ok_or(anyhow!("stderr was piped"))?;
        wait_for_output(stderr, "connections", self.verbose)?;

        self.process = Some(child);
        tracing::info!("Postgres {} started on port {}", self.name, self.port);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), anyhow::Error> {
        if self.process.is_none() {
            return Ok(());
        }
        let mut child = self.process.take().unwrap();
        tracing::info!("Stopping {} postgres...", self.name);
        child.kill()?;
        child.wait()?;
        tracing::info!("Postgres {} stopped", self.name);
        Ok(())
    }
}

impl Drop for Postgres {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            tracing::error!("Failed to stop postgres: {}", e);
        }
    }
}
