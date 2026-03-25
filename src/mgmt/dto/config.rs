use std::env::current_dir;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
    pub(crate) port: u16,
    pub(crate) jwt_secret: String,
    pub(crate) daemon_directory: PathBuf,
    pub(crate) binaries_directory: PathBuf,
}

impl Config {
    pub fn new() -> Result<Self, anyhow::Error> {
        let port: u16 = std::env::var("PORT")?.parse()?;
        let jwt_secret = std::env::var("JWT_SECRET")?;
        let daemon_directory = current_dir()?.join("neon_daemon_data");
        let binaries_directory = tempfile::TempDir::new()?.keep();
        Ok(Self {
            port,
            jwt_secret,
            daemon_directory,
            binaries_directory,
        })
    }
}
