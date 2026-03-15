mod death;
mod stdout;

use crate::daemon::stdout::wait_for_output;
use anyhow::anyhow;
use std::ffi::OsStr;
use std::path::{PathBuf};
use std::process::{Child, Command, Stdio};

pub struct Daemon {
    daemon_directory: PathBuf,
    storage_broker_process: Option<Child>,
    verbose: bool,
}

impl Daemon {
    pub fn new(daemon_directory: PathBuf) -> Self {
        Daemon {
            daemon_directory,
            storage_broker_process: None,
            verbose: cfg!(debug_assertions),
        }
    }

    pub fn start(&mut self) -> Result<(), anyhow::Error> {
        self.start_storage_broker()?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Stopping daemon...");
        self.stop_storage_broker()?;
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
                libc::killpg(child.id() as i32, libc::SIGTERM);
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
            child.kill().ok();
            child.wait().ok();
        }
        Ok(())
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            tracing::error!("Failed to stop daemon: {}", e);
        }
    }
}
