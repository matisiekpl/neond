use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::mgmt::dto::config::{Config, RemoteStorageConfig};
use crate::mgmt::dto::error::{AppError, Result};

pub struct BackupService {
    daemon_directory: PathBuf,
    pg_install_directory: PathBuf,
    server_secret: String,
    remote_storage_config: Option<RemoteStorageConfig>,
    s3_client: Option<aws_sdk_s3::Client>,
    interval: Duration,
    shutdown_token: CancellationToken,
    task: Mutex<Option<JoinHandle<()>>>,
}

impl BackupService {
    pub async fn new(config: Config, shutdown_token: CancellationToken) -> Result<Self> {
        let s3_client = match &config.remote_storage_config {
            Some(remote) => {
                let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
                    .region(aws_sdk_s3::config::Region::new(remote.region.clone()))
                    .load()
                    .await;
                Some(aws_sdk_s3::Client::new(&aws_config))
            }
            None => None,
        };
        Ok(Self {
            daemon_directory: config.daemon_directory.clone(),
            pg_install_directory: config.pg_install_directory.clone(),
            server_secret: config.server_secret.clone(),
            remote_storage_config: config.remote_storage_config.clone(),
            s3_client,
            interval: config.backup_interval,
            shutdown_token,
            task: Mutex::new(None),
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.s3_client.is_some()
    }

    fn object_key(database_name: &str) -> String {
        format!("control-plane/{}/latest.tar.gz", database_name)
    }

    pub async fn restore_if_needed(
        &self,
        database_name: &str,
        data_directory: &Path,
    ) -> Result<()> {
        let client = match &self.s3_client {
            Some(client) => client,
            None => return Ok(()),
        };
        let bucket = match &self.remote_storage_config {
            Some(remote) => &remote.bucket,
            None => return Ok(()),
        };

        if data_directory.join("postgresql.conf").exists() {
            tracing::info!(
                "Local data directory for {} already initialized, skipping restore",
                database_name
            );
            return Ok(());
        }

        let key = Self::object_key(database_name);
        let exists = client
            .head_object()
            .bucket(bucket)
            .key(&key)
            .send()
            .await
            .is_ok();
        if !exists {
            tracing::info!(
                "No backup for {} found in s3://{}/{}, skipping restore",
                database_name, bucket, key
            );
            return Ok(());
        }

        tracing::info!("Restoring {} from s3://{}/{}", database_name, bucket, key);

        std::fs::create_dir_all(&self.daemon_directory).map_err(|error| {
            AppError::BackupRestoreFailed {
                database_name: database_name.to_string(),
                reason: format!("create daemon directory: {}", error),
            }
        })?;
        let tempdir = tempfile::TempDir::new_in(&self.daemon_directory).map_err(|error| {
            AppError::BackupRestoreFailed {
                database_name: database_name.to_string(),
                reason: format!("create tempdir: {}", error),
            }
        })?;
        let archive_path = tempdir.path().join("base.tar.gz");

        let response = client
            .get_object()
            .bucket(bucket)
            .key(&key)
            .send()
            .await
            .map_err(|error| AppError::BackupDownloadFailed {
                database_name: database_name.to_string(),
                reason: error.to_string(),
            })?;
        let bytes = response
            .body
            .collect()
            .await
            .map_err(|error| AppError::BackupDownloadFailed {
                database_name: database_name.to_string(),
                reason: format!("read body: {}", error),
            })?
            .into_bytes();
        tokio::fs::write(&archive_path, &bytes).await.map_err(|error| {
            AppError::BackupDownloadFailed {
                database_name: database_name.to_string(),
                reason: format!("write archive: {}", error),
            }
        })?;

        std::fs::create_dir_all(data_directory).map_err(|error| {
            AppError::BackupRestoreFailed {
                database_name: database_name.to_string(),
                reason: format!("create data directory: {}", error),
            }
        })?;

        let archive_path_blocking = archive_path.clone();
        let data_directory_blocking = data_directory.to_path_buf();
        tokio::task::spawn_blocking(
            move || -> std::result::Result<(), std::io::Error> {
                let file = std::fs::File::open(&archive_path_blocking)?;
                let decoder = flate2::read::GzDecoder::new(file);
                let mut archive = tar::Archive::new(decoder);
                archive.set_preserve_permissions(true);
                archive.unpack(&data_directory_blocking)?;
                Ok(())
            },
        )
        .await
        .map_err(|error| AppError::BackupRestoreFailed {
            database_name: database_name.to_string(),
            reason: format!("join extract task: {}", error),
        })?
        .map_err(|error| AppError::BackupRestoreFailed {
            database_name: database_name.to_string(),
            reason: format!("extract archive: {}", error),
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(data_directory)
                .map_err(|error| AppError::BackupRestoreFailed {
                    database_name: database_name.to_string(),
                    reason: format!("read metadata: {}", error),
                })?
                .permissions();
            perms.set_mode(0o700);
            std::fs::set_permissions(data_directory, perms).map_err(|error| {
                AppError::BackupRestoreFailed {
                    database_name: database_name.to_string(),
                    reason: format!("set permissions: {}", error),
                }
            })?;
        }

        tracing::info!(
            "Restored {} from s3://{}/{} to {}",
            database_name,
            bucket,
            key,
            data_directory.display()
        );
        Ok(())
    }

    pub async fn sync(&self, database_name: &str, port: u16) -> Result<()> {
        let client = match &self.s3_client {
            Some(client) => client,
            None => return Ok(()),
        };
        let bucket = match &self.remote_storage_config {
            Some(remote) => &remote.bucket,
            None => return Ok(()),
        };

        std::fs::create_dir_all(&self.daemon_directory).map_err(|error| {
            AppError::BackupUploadFailed {
                database_name: database_name.to_string(),
                reason: format!("create daemon directory: {}", error),
            }
        })?;
        let tempdir = tempfile::TempDir::new_in(&self.daemon_directory).map_err(|error| {
            AppError::BackupUploadFailed {
                database_name: database_name.to_string(),
                reason: format!("create tempdir: {}", error),
            }
        })?;
        let archive_path = tempdir.path().join("base.tar.gz");

        let pg_basebackup_binary =
            self.pg_install_directory.join("vanilla_v17/bin/pg_basebackup");
        let pg_lib = self.pg_install_directory.join("vanilla_v17/lib");

        let status = tokio::process::Command::new(&pg_basebackup_binary)
            .env("DYLD_LIBRARY_PATH", &pg_lib)
            .env("LD_LIBRARY_PATH", &pg_lib)
            .env("PGPASSWORD", &self.server_secret)
            .arg("-h")
            .arg("127.0.0.1")
            .arg("-p")
            .arg(port.to_string())
            .arg("-U")
            .arg("neond")
            .arg("-D")
            .arg(tempdir.path())
            .arg("-Ft")
            .arg("-z")
            .arg("-X")
            .arg("fetch")
            .arg("--no-sync")
            .kill_on_drop(true)
            .status()
            .await
            .map_err(|error| AppError::BackupUploadFailed {
                database_name: database_name.to_string(),
                reason: format!("spawn pg_basebackup: {}", error),
            })?;
        if !status.success() {
            return Err(AppError::BackupUploadFailed {
                database_name: database_name.to_string(),
                reason: format!("pg_basebackup exited with {}", status),
            });
        }

        let key = Self::object_key(database_name);
        let body = aws_sdk_s3::primitives::ByteStream::from_path(&archive_path)
            .await
            .map_err(|error| AppError::BackupUploadFailed {
                database_name: database_name.to_string(),
                reason: format!("read archive: {}", error),
            })?;
        client
            .put_object()
            .bucket(bucket)
            .key(&key)
            .body(body)
            .send()
            .await
            .map_err(|error| AppError::BackupUploadFailed {
                database_name: database_name.to_string(),
                reason: error.to_string(),
            })?;
        tracing::info!(
            "Uploaded {} backup to s3://{}/{}",
            database_name,
            bucket,
            key
        );
        Ok(())
    }

    pub async fn start_periodic(self: Arc<Self>, databases: Vec<(&'static str, u16)>) {
        if !self.is_enabled() {
            return;
        }
        let interval = self.interval;
        let shutdown_token = self.shutdown_token.clone();
        let service = Arc::clone(&self);
        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = shutdown_token.cancelled() => {
                        tracing::info!("Periodic backup loop received shutdown signal");
                        break;
                    }
                    _ = ticker.tick() => {
                        for (database_name, port) in &databases {
                            if let Err(error) = service.sync(database_name, *port).await {
                                tracing::warn!(
                                    "Periodic backup for {} failed: {}",
                                    database_name, error
                                );
                            }
                        }
                    }
                }
            }
        });
        let mut task = self.task.lock().await;
        *task = Some(handle);
        tracing::info!(
            "Periodic backup loop started with interval {:?}",
            interval
        );
    }

    pub async fn stop_periodic(&self) {
        let handle_option = {
            let mut task = self.task.lock().await;
            task.take()
        };
        if let Some(handle) = handle_option {
            let _ = handle.await;
            tracing::info!("Periodic backup loop stopped");
        }
    }

    pub async fn final_sync(&self, databases: &[(&'static str, u16)]) {
        if !self.is_enabled() {
            return;
        }
        tracing::info!("Running final backup before shutdown...");
        for (database_name, port) in databases {
            if let Err(error) = self.sync(database_name, *port).await {
                tracing::error!(
                    "Final backup for {} failed: {}",
                    database_name, error
                );
            }
        }
    }
}