use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use aws_sdk_s3::primitives::ByteStream;

use crate::mgmt::dto::config::RemoteStorageConfig;

const LEASE_KEY: &str = "control-plane/.lock";
const LOCAL_LOCK_FILENAME: &str = ".lock";

#[derive(Debug)]
pub enum LeaseError {
    LocalAlreadyHeld { path: PathBuf },
    S3AlreadyHeld { bucket: String, key: String },
    LocalIo { reason: String },
    S3Error { reason: String },
}

impl std::fmt::Display for LeaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeaseError::LocalAlreadyHeld { path } => {
                write!(f, "local lock file already exists at {}", path.display())
            }
            LeaseError::S3AlreadyHeld { bucket, key } => {
                write!(f, "S3 lock object already exists at s3://{}/{}", bucket, key)
            }
            LeaseError::LocalIo { reason } => write!(f, "local lock io error: {}", reason),
            LeaseError::S3Error { reason } => write!(f, "S3 lock error: {}", reason),
        }
    }
}

impl std::error::Error for LeaseError {}

struct S3Component {
    client: aws_sdk_s3::Client,
    bucket: String,
}

pub struct DaemonLease {
    local_lock_path: PathBuf,
    local_lock_file: Option<File>,
    s3: Option<S3Component>,
}

impl DaemonLease {
    pub async fn acquire(
        daemon_directory: &Path,
        remote_storage_config: Option<&RemoteStorageConfig>,
    ) -> Result<Self, LeaseError> {
        let local_lock_path = daemon_directory.join(LOCAL_LOCK_FILENAME);
        let local_lock_file = acquire_local_lock(&local_lock_path)?;

        let s3 = match remote_storage_config {
            Some(remote) => Some(acquire_s3_lock(remote).await.map_err(|error| {
                let _ = std::fs::remove_file(&local_lock_path);
                error
            })?),
            None => None,
        };

        Ok(Self {
            local_lock_path,
            local_lock_file: Some(local_lock_file),
            s3,
        })
    }

    pub async fn release(mut self) {
        if let Some(s3) = self.s3.take() {
            match s3
                .client
                .delete_object()
                .bucket(&s3.bucket)
                .key(LEASE_KEY)
                .send()
                .await
            {
                Ok(_) => tracing::info!(
                    "Daemon lease released (s3://{}/{})",
                    s3.bucket,
                    LEASE_KEY
                ),
                Err(error) => tracing::warn!(
                    "Failed to delete S3 lock object s3://{}/{}: {}. You may need to remove it manually.",
                    s3.bucket,
                    LEASE_KEY,
                    error
                ),
            }
        }

        self.local_lock_file.take();
        match std::fs::remove_file(&self.local_lock_path) {
            Ok(_) => tracing::info!(
                "Daemon lease released ({})",
                self.local_lock_path.display()
            ),
            Err(error) => tracing::warn!(
                "Failed to delete local lock file {}: {}. You may need to remove it manually.",
                self.local_lock_path.display(),
                error
            ),
        }
    }
}

fn acquire_local_lock(path: &Path) -> Result<File, LeaseError> {
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(file) => Ok(file),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            Err(LeaseError::LocalAlreadyHeld {
                path: path.to_path_buf(),
            })
        }
        Err(error) => Err(LeaseError::LocalIo {
            reason: error.to_string(),
        }),
    }
}

async fn acquire_s3_lock(
    remote_storage_config: &RemoteStorageConfig,
) -> Result<S3Component, LeaseError> {
    let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(aws_sdk_s3::config::Region::new(
            remote_storage_config.region.clone(),
        ))
        .load()
        .await;
    let client = aws_sdk_s3::Client::new(&aws_config);

    let put_result = client
        .put_object()
        .bucket(&remote_storage_config.bucket)
        .key(LEASE_KEY)
        .if_none_match("*")
        .body(ByteStream::from_static(b"locked"))
        .send()
        .await;

    match put_result {
        Ok(_) => Ok(S3Component {
            client,
            bucket: remote_storage_config.bucket.clone(),
        }),
        Err(error) => {
            let status = error.raw_response().map(|response| response.status().as_u16());
            if status == Some(412) {
                Err(LeaseError::S3AlreadyHeld {
                    bucket: remote_storage_config.bucket.clone(),
                    key: LEASE_KEY.to_string(),
                })
            } else {
                Err(LeaseError::S3Error {
                    reason: error.to_string(),
                })
            }
        }
    }
}