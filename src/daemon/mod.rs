use std::path::PathBuf;
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
        self.stop_storage_broker()?;
        Ok(())
    }

    fn start_storage_broker(&mut self) -> Result<(), anyhow::Error> {
        let storage_broker_path = self.daemon_directory.join("binaries/storage_broker");

        let child = Command::new(storage_broker_path)
            .env_clear()
            .args(["-l", "127.0.0.1:50051"])
            .stdout(if self.verbose {
                Stdio::inherit()
            } else {
                Stdio::null()
            })
            .spawn()?;
        let pid = child.id();
        self.storage_broker_process = Some(child);
        tracing::info!(
            "Storage broker started started on port 50051 on PID: {}",
            pid,
        );
        Ok(())
    }

    fn stop_storage_broker(&mut self) -> Result<(), anyhow::Error> {
        if let Some(mut child) = self.storage_broker_process.take() {
            child.kill()?;
        }
        Ok(())
    }
}
