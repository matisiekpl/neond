use crate::daemon::death;
use crate::daemon::stdout::wait_for_output;
use anyhow::anyhow;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

pub struct ProcessControl {
    name: String,
    binary: PathBuf,
    args: Vec<OsString>,
    working_directory: PathBuf,
    needle: String,
    verbose: bool,
    child: Option<Child>,
}

impl ProcessControl {
    pub fn new(
        name: impl Into<String>,
        binary: PathBuf,
        args: impl IntoIterator<Item = impl Into<OsString>>,
        working_directory: PathBuf,
        needle: impl Into<String>,
        verbose: bool,
    ) -> Self {
        ProcessControl {
            name: name.into(),
            binary,
            args: args.into_iter().map(|a| a.into()).collect(),
            working_directory,
            needle: needle.into(),
            verbose,
            child: None,
        }
    }

    pub fn start(&mut self) -> Result<(), anyhow::Error> {
        let mut cmd = Command::new(&self.binary);
        death::configure_death_signal(&mut cmd);
        let mut child = cmd
            .env_clear()
            .current_dir(&self.working_directory)
            .args(&self.args)
            .stdout(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().ok_or(anyhow!("stdout was piped"))?;
        wait_for_output(stdout, &self.needle, self.verbose, self.verbose)?;

        let pid = child.id();
        self.child = Some(child);
        tracing::info!("{} started on PID: {}", self.name, pid);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), anyhow::Error> {
        if let Some(mut child) = self.child.take() {
            #[cfg(unix)]
            unsafe {
                tracing::debug!("Sending SIGINT to process: {}", child.id());
                libc::killpg(child.id() as i32, libc::SIGINT);
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
            child.kill().ok();
            child.wait().ok();
        }
        tracing::info!("{} stopped", self.name);
        Ok(())
    }
}
