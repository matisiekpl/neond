use std::fs;
use std::path::{Path, PathBuf};

use neon_utils::auth::{Claims, Scope, encode_from_key_file};
use neon_utils::id::TenantId;
use openssl::pkey::PKey;
use tempfile::TempDir;

use crate::mgmt::dto::error::{AppError, Result};

pub struct DaemonAuth {
    private_key: pem::Pem,
    public_key_path: PathBuf,
    #[allow(dead_code)]
    keys_dir: TempDir,
}

impl DaemonAuth {
    pub fn generate() -> Result<Self> {
        let keys_dir =
            TempDir::with_prefix("neond_auth_").map_err(|error| AppError::AuthKeyGenerationFailed {
                reason: error.to_string(),
            })?;

        let key = PKey::generate_ed25519().map_err(|error| AppError::AuthKeyGenerationFailed {
            reason: error.to_string(),
        })?;

        let private_key_der =
            key.private_key_to_der()
                .map_err(|error| AppError::AuthKeyGenerationFailed {
                    reason: error.to_string(),
                })?;
        let private_key = pem::Pem::new("PRIVATE KEY", private_key_der);

        let public_key_pem =
            key.public_key_to_pem()
                .map_err(|error| AppError::AuthKeyGenerationFailed {
                    reason: error.to_string(),
                })?;
        let public_key_path = keys_dir.path().join("auth_public_key.pem");
        fs::write(&public_key_path, &public_key_pem).map_err(|error| {
            AppError::AuthKeyGenerationFailed {
                reason: error.to_string(),
            }
        })?;

        tracing::info!(
            "Generated ephemeral Ed25519 auth keys in {}",
            keys_dir.path().display()
        );

        Ok(Self {
            private_key,
            public_key_path,
            keys_dir,
        })
    }

    pub fn generate_token(
        &self,
        scope: Scope,
        tenant_id: Option<TenantId>,
    ) -> Result<String> {
        let claims = Claims::new(tenant_id, scope);
        encode_from_key_file(&claims, &self.private_key).map_err(|error| {
            AppError::TokenGenerationFailed {
                reason: error.to_string(),
            }
        })
    }

    pub fn public_key_path(&self) -> &Path {
        &self.public_key_path
    }

    pub fn public_key_pem_string(&self) -> Result<String> {
        fs::read_to_string(&self.public_key_path).map_err(|error| AppError::AuthKeyUnreadable {
            path: self.public_key_path.display().to_string(),
            reason: error.to_string(),
        })
    }
}
