use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::service::logs::{LogChannel, LogStream, LogsService};
use crate::utils::death;
use crate::utils::stdout::wait_for_output_timeout;
use nix::sys::signal::{Signal::SIGINT, kill};
use nix::unistd::Pid;
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tempfile::NamedTempFile;

pub struct Postgres {
    name: &'static str,
    initdb_binary_path: PathBuf,
    postgres_binary_path: PathBuf,
    postgres_lib_path: PathBuf,
    data_directory: PathBuf,
    port: u16,
    process: Option<std::process::Child>,
    verbose: bool,
    password: String,
    logs_service: Option<Arc<LogsService>>,
    log_channel: Option<LogChannel>,
}

impl Postgres {
    pub fn new(
        name: &'static str,
        daemon_directory: PathBuf,
        pg_install_directory: PathBuf,
        data_directory_suffix: &'static str,
        port: u16,
        password: String,
    ) -> Self {
        Postgres {
            name,
            initdb_binary_path: pg_install_directory.join("vanilla_v17/bin/initdb"),
            postgres_binary_path: pg_install_directory.join("vanilla_v17/bin/postgres"),
            postgres_lib_path: pg_install_directory.join("vanilla_v17/lib"),
            data_directory: daemon_directory
                .join("daemon_data")
                .join(data_directory_suffix),
            port,
            process: None,
            verbose: cfg!(debug_assertions),
            password,
            logs_service: None,
            log_channel: None,
        }
    }

    pub fn with_logs(mut self, logs_service: Arc<LogsService>, channel: LogChannel) -> Self {
        self.logs_service = Some(logs_service);
        self.log_channel = Some(channel);
        self
    }

    pub fn init(&self) -> Result<()> {
        if self.data_directory.join("postgresql.conf").exists() {
            tracing::info!("Postgres data dir for {} already initialized", self.name);
            return Ok(());
        }

        let mut pwfile = NamedTempFile::new().map_err(|error| {
            AppError::PostgresInitializationFailed {
                reason: error.to_string(),
            }
        })?;
        write!(pwfile, "{}", self.password).map_err(|error| {
            AppError::PostgresInitializationFailed {
                reason: error.to_string(),
            }
        })?;
        pwfile
            .flush()
            .map_err(|error| AppError::PostgresInitializationFailed {
                reason: error.to_string(),
            })?;

        let exit_status = std::process::Command::new(self.initdb_binary_path.clone())
            .env("DYLD_LIBRARY_PATH", &self.postgres_lib_path)
            .env("LD_LIBRARY_PATH", &self.postgres_lib_path)
            .arg("-U")
            .arg("neond")
            .arg("--pwfile")
            .arg(pwfile.path())
            .arg("--auth-local=scram-sha-256")
            .arg("--auth-host=scram-sha-256")
            .arg("-D")
            .arg(&self.data_directory)
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|error| AppError::PostgresInitializationFailed {
                reason: error.to_string(),
            })?
            .wait()
            .map_err(|error| AppError::PostgresInitializationFailed {
                reason: error.to_string(),
            })?;
        let exit_code =
            exit_status
                .code()
                .ok_or_else(|| AppError::PostgresInitializationFailed {
                    reason: "initdb terminated by signal".to_string(),
                })?;
        if exit_code != 0 {
            return Err(AppError::PostgresInitializationFailed {
                reason: format!("initdb exited with code {}", exit_code),
            });
        }
        tracing::info!("Postgres data dir for {} initialized", self.name);
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        let mut cmd = std::process::Command::new(self.postgres_binary_path.clone());
        death::configure_death_signal(&mut cmd);
        let mut child = cmd
            .env("DYLD_LIBRARY_PATH", &self.postgres_lib_path)
            .env("LD_LIBRARY_PATH", &self.postgres_lib_path)
            .arg("-D")
            .arg(&self.data_directory)
            .arg("-p")
            .arg(self.port.to_string())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| AppError::PostgresStartupFailed {
                reason: error.to_string(),
            })?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| AppError::PostgresStartupFailed {
                reason: "stderr was not piped".to_string(),
            })?;

        let stderr_sink: Option<Box<dyn Fn(String) + Send + 'static>> =
            self.logs_service.as_ref().map(|logs| {
                let logs = Arc::clone(logs);
                let channel = self.log_channel.clone().unwrap();
                Box::new(move |line: String| {
                    logs.ingest(channel.clone(), line, LogStream::Stderr);
                }) as Box<dyn Fn(String) + Send + 'static>
            });

        wait_for_output_timeout(stderr, "connections", self.verbose, self.verbose, None, stderr_sink).map_err(|error| {
            AppError::PostgresStartupFailed {
                reason: error.to_string(),
            }
        })?;

        self.process = Some(child);
        tracing::info!("Postgres {} started on port {}", self.name, self.port);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        let mut child = match self.process.take() {
            Some(child) => child,
            None => return Ok(()),
        };
        tracing::info!("Stopping {} postgres...", self.name);
        let pid = Pid::from_raw(child.id() as i32);
        kill(pid, SIGINT).map_err(|error| AppError::PostgresShutdownFailed {
            reason: error.to_string(),
        })?;
        child
            .wait()
            .map_err(|error| AppError::PostgresShutdownFailed {
                reason: error.to_string(),
            })?;
        tracing::info!("Postgres {} stopped", self.name);

        Ok(())
    }

    pub fn get_connection_uri(&self) -> String {
        format!(
            "postgresql://neond:{}@localhost:{}/postgres",
            self.password, self.port
        )
    }

    pub fn data_directory(&self) -> &std::path::Path {
        &self.data_directory
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for Postgres {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            tracing::error!("Failed to stop postgres: {}", e);
        }
    }
}
