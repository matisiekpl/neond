mod death;
mod postgres;
mod stdout;
mod tracer;

use crate::daemon::stdout::wait_for_output;
use anyhow::anyhow;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

pub struct Daemon {
    daemon_directory: PathBuf,
    storage_broker_process: Option<Child>,
    storage_controller_process: Option<Child>,
    verbose: bool,
    storage_controller_postgres: postgres::Postgres,
    management_postgres: postgres::Postgres,
    tracer: tracer::Tracer,

    pageserver_working_directory: PathBuf,
    safekeeper_working_directory: PathBuf,

    pageserver_process: Option<Child>,
    safekeeper_process: Option<Child>,
}

impl Daemon {
    pub fn new(daemon_directory: PathBuf) -> Self {
        Daemon {
            daemon_directory: daemon_directory.clone(),
            storage_broker_process: None,
            storage_controller_process: None,
            verbose: cfg!(debug_assertions),
            storage_controller_postgres: postgres::Postgres::new(
                "storage_controller_db",
                daemon_directory.clone(),
                "storage_controller_pg_data",
                5431,
                "mateuszek".to_string(),
            ),
            management_postgres: postgres::Postgres::new(
                "management_db",
                daemon_directory.clone(),
                "management_pg_data",
                5430,
                "mateuszek".to_string(),
            ),
            tracer: tracer::Tracer::new(),

            pageserver_working_directory: daemon_directory.join("pageserver"),
            safekeeper_working_directory: daemon_directory.join("safekeeper"),

            pageserver_process: None,
            safekeeper_process: None,
        }
    }

    pub fn start(&mut self) -> Result<(), anyhow::Error> {
        self.storage_controller_postgres.init()?;
        self.management_postgres.init()?;
        self.storage_controller_postgres.start()?;
        self.management_postgres.start()?;
        self.tracer.start();
        self.start_storage_broker()?;
        self.start_storage_controller()?;
        self.start_safekeeper()?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Stopping daemon...");
        self.stop_safekeeper()?;
        self.tracer.stop();
        self.stop_storage_broker()?;
        self.stop_storage_controller()?;
        self.storage_controller_postgres.stop()?;
        self.management_postgres.stop()?;
        Ok(())
    }

    fn start_storage_broker(&mut self) -> Result<(), anyhow::Error> {
        let storage_broker_path = self.daemon_directory.join("binaries/storage_broker");

        let child = Self::start_process(
            storage_broker_path,
            ["-l", "127.0.0.1:50051"],
            "listening",
            self.verbose,
        )?;
        let pid = child.id();
        self.storage_broker_process = Some(child);

        tracing::info!("Storage broker started on port 50051 on PID: {}", pid,);
        Ok(())
    }

    fn stop_storage_broker(&mut self) -> Result<(), anyhow::Error> {
        Self::stop_process(&mut self.storage_broker_process)?;
        tracing::info!("Storage broker stopped");
        Ok(())
    }

    fn start_storage_controller(&mut self) -> Result<(), anyhow::Error> {
        let storage_controller_path = self.daemon_directory.join("binaries/storage_controller");
        let child = Self::start_process(
            storage_controller_path,
            [
                "-l",
                "127.0.0.1:1234",
                "--database-url",
                self.storage_controller_postgres
                    .get_connection_uri()
                    .as_str(),
                "--dev",
                "--timeline-safekeeper-count",
                "1",
                "--control-plane-url",
                "http://127.0.0.1:1235",
            ],
            "Serving HTTP on 127.0.0.1:1234",
            self.verbose,
        )?;

        let pid = child.id();
        self.storage_controller_process = Some(child);
        tracing::info!("Storage controller started on port 1234 on PID: {}", pid);
        Ok(())
    }

    fn stop_storage_controller(&mut self) -> Result<(), anyhow::Error> {
        Self::stop_process(&mut self.storage_controller_process)?;
        tracing::info!("Storage controller stopped");

        Ok(())
    }

    fn start_safekeeper(&mut self) -> Result<(), anyhow::Error> {
        std::fs::create_dir_all(&self.safekeeper_working_directory)?;

        let child = Self::start_process(
            self.daemon_directory.join("binaries/safekeeper"),
            [
                "-D",
                self.safekeeper_working_directory.to_str().unwrap(),
                "--id",
                "1",
                "--broker-endpoint",
                "http://127.0.0.1:50051",
                "--listen-pg",
                "127.0.0.1:5454",
                "--listen-http",
                "127.0.0.1:7676",
                "--availability-zone",
                "primary",
            ],
            "starting safekeeper WAL service on",
            self.verbose,
        )?;

        let pid = child.id();
        tracing::info!("Safekeeper started on PID: {}", pid);

        self.safekeeper_process = Some(child);
        Ok(())
    }

    fn stop_safekeeper(&mut self) -> Result<(), anyhow::Error> {
        Self::stop_process(&mut self.safekeeper_process)?;
        tracing::info!("Safekeeper stopped");
        Ok(())
    }

    fn start_process(
        binary: PathBuf,
        args: impl IntoIterator<Item = impl AsRef<OsStr>>,
        needle: &str,
        verbose: bool,
    ) -> Result<Child, anyhow::Error> {
        let mut cmd = Command::new(binary);
        death::configure_death_signal(&mut cmd);
        let mut child = cmd.env_clear().args(args).stdout(Stdio::piped()).spawn()?;

        let stdout = child.stdout.take().ok_or(anyhow!("stdout was piped"))?;
        wait_for_output(stdout, needle, verbose)?;

        Ok(child)
    }

    fn stop_process(child: &mut Option<Child>) -> Result<(), anyhow::Error> {
        if let Some(mut child) = child.take() {
            #[cfg(unix)]
            unsafe {
                tracing::debug!("Sending SIGINT to process: {}", child.id());
                libc::killpg(child.id() as i32, libc::SIGINT);
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
            child.kill().ok();
            child.wait().ok();
        }
        Ok(())
    }

    pub fn get_management_postgres_uri(&self) -> String {
        self.management_postgres.get_connection_uri()
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            tracing::error!("Failed to stop daemon: {}", e);
        }
    }
}
