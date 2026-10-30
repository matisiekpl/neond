use aws_sdk_s3::primitives::ByteStream;

use crate::mgmt::dto::config::RemoteStorageConfig;

use super::PreflightError;

pub async fn check_s3_write_access(config: &RemoteStorageConfig) -> Result<(), PreflightError> {
    let client = config.s3_client().await;
    let key = ".neond-preflight-check";

    client
        .put_object()
        .bucket(&config.bucket)
        .key(key)
        .body(ByteStream::from_static(b"preflight"))
        .send()
        .await
        .map_err(|e| {
            PreflightError::S3WriteCheckFailed(format!(
                "Failed to write to s3://{}/{}: {}",
                config.bucket, key, e
            ))
        })?;

    let _ = client
        .delete_object()
        .bucket(&config.bucket)
        .key(key)
        .send()
        .await;

    Ok(())
}