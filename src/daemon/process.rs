use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::service::logs::{LogChannel, LogStream, LogsService};
use crate::utils::death;
use crate::utils::stdout::wait_for_output_timeout;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;

pub struct ProcessControl {
    name: String,
    binary: PathBuf,
    args: Vec<OsString>,
    env_vars: Vec<(String, String)>,
    working_directory: PathBuf,
    needle: String,
    verbose: bool,
    child: Option<Child>,
    logs_service: Option<Arc<LogsService>>,
    log_channel: Option<LogChannel>,
}

impl ProcessControl {
    pub fn new(
        name: impl Into<String>,
        binary: PathBuf,
        args: impl IntoIterator<Item = impl Into<OsString>>,
        env_vars: Vec<(String, String)>,
        working_directory: PathBuf,
        needle: impl Into<String>,
        verbose: bool,
    ) -> Self {
        ProcessControl {
            name: name.into(),
            binary,
            args: args.into_iter().map(|a| a.into()).collect(),
            env_vars,
            working_directory,
            needle: needle.into(),
            verbose,
            child: None,
            logs_service: None,
            log_channel: None,
        }
    }

    pub fn with_logs(mut self, logs_service: Arc<LogsService>, channel: LogChannel) -> Self {
        self.logs_service = Some(logs_service);
        self.log_channel = Some(channel);
        self
    }

    pub fn start(&mut self) -> Result<()> {
        let mut cmd = Command::new(&self.binary);
        death::configure_death_signal(&mut cmd);
        let mut child = cmd
            .current_dir(&self.working_directory)
            .args(&self.args)
            .envs(self.env_vars.iter().map(|(k, v)| (k, v)))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| AppError::DaemonStartupFailed {
                reason: format!("failed to spawn {}: {}", self.name, error),
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::DaemonStartupFailed {
                reason: format!("stdout was not piped for {}", self.name),
            })?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| AppError::DaemonStartupFailed {
                reason: format!("stderr was not piped for {}", self.name),
            })?;

        let stdout_sink: Option<Box<dyn Fn(String) + Send + 'static>> =
            self.logs_service.as_ref().map(|logs| {
                let logs = Arc::clone(logs);
                let channel = self.log_channel.clone().unwrap();
                Box::new(move |line: String| {
                    logs.ingest(channel.clone(), line, LogStream::Stdout);
                }) as Box<dyn Fn(String) + Send + 'static>
            });

        let stderr_sink: Option<Box<dyn Fn(String) + Send + 'static>> =
            self.logs_service.as_ref().map(|logs| {
                let logs = Arc::clone(logs);
                let channel = self.log_channel.clone().unwrap();
                Box::new(move |line: String| {
                    logs.ingest(channel.clone(), line, LogStream::Stderr);
                }) as Box<dyn Fn(String) + Send + 'static>
            });

        std::thread::spawn(move || {
            if let Some(sink) = stderr_sink {
                let reader = std::io::BufReader::new(stderr);
                use std::io::BufRead;
                for line in reader.lines() {
                    if let Ok(line) = line {
                        sink(line);
                    }
                }
            }
        });

        wait_for_output_timeout(stdout, &self.needle, self.verbose, self.verbose, None, stdout_sink).map_err(|error| {
            AppError::DaemonStartupFailed {
                reason: format!("waiting on {} stdout: {}", self.name, error),
            }
        })?;

        let pid = child.id();
        self.child = Some(child);
        tracing::info!("{} started on PID: {}", self.name, pid);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            #[cfg(unix)]
            {
                let pid = child.id() as i32;
                tracing::debug!("Sending SIGINT to process group: {}", pid);
                unsafe {
                    libc::killpg(pid, libc::SIGINT);
                }
                for _ in 0..50 {
                    match child.try_wait() {
                        Ok(Some(_)) => {
                            tracing::info!("{} stopped gracefully", self.name);
                            return Ok(());
                        }
                        Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                        Err(e) => {
                            tracing::warn!("Error waiting for {}: {}", self.name, e);
                            break;
                        }
                    }
                }
                tracing::warn!("{} did not stop gracefully, sending SIGKILL", self.name);
            }
            child.kill().ok();
            child.wait().ok();
        }
        tracing::info!("{} stopped", self.name);
        Ok(())
    }
}
