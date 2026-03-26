use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use neon_utils::auth::{Claims, Scope, encode_from_key_file};
use neon_utils::id::TenantId;
use openssl::pkey::PKey;
use tempfile::TempDir;

pub struct DaemonAuth {
    private_key: pem::Pem,
    public_key_path: PathBuf,
    #[allow(dead_code)]
    keys_dir: TempDir,
}

impl DaemonAuth {
    pub fn generate() -> Result<Self> {
        let keys_dir = TempDir::with_prefix("neond_auth_")?;

        let key = PKey::generate_ed25519()?;

        let private_key_der = key.private_key_to_der()?;
        let private_key = pem::Pem::new("PRIVATE KEY", private_key_der);

        let public_key_pem = key.public_key_to_pem()?;
        let public_key_path = keys_dir.path().join("auth_public_key.pem");
        fs::write(&public_key_path, &public_key_pem)?;

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

    pub fn generate_token(&self, scope: Scope, tenant_id: Option<TenantId>) -> String {
        let claims = Claims::new(tenant_id, scope);
        encode_from_key_file(&claims, &self.private_key).expect("failed to generate JWT token")
    }

    pub fn public_key_path(&self) -> &Path {
        &self.public_key_path
    }

    pub fn public_key_pem_string(&self) -> String {
        fs::read_to_string(&self.public_key_path).expect("failed to read public key")
    }
}
