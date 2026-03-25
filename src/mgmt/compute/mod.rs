use anyhow::anyhow;
use base64::prelude::*;
use hmac::{Hmac, Mac};
use neon_compute_api::responses::{ComputeConfig, ComputeCtlConfig};
use neon_compute_api::spec::{
    Cluster, ComputeAudit, ComputeMode, ComputeSpec, Database, PageserverConnectionInfo,
    PageserverShardConnectionInfo, PageserverShardInfo, PgIdent, Role,
};
use neon_utils::id::{NodeId, TenantId, TimelineId};
use neon_utils::shard::{ShardCount, ShardIndex};
use pbkdf2::pbkdf2_hmac;
use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnValue, ExtendedKeyUsagePurpose, IsCa,
    Issuer, KeyPair, KeyUsagePurpose,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::str::FromStr;
use tempfile::TempDir;

use crate::utils::stdout::wait_for_output_timeout;

use crate::mgmt::model::branch::Branch;
use crate::mgmt::model::project::PgVersion;
use neon_control_plane::postgresql_conf::PostgresConf;
use postgres_protocol::authentication::sasl::{ChannelBinding, ScramSha256};

use base64::{Engine as _, engine::general_purpose};
use rand::RngCore;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComputeEndpointStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

pub struct ComputeEndpoint {
    branch: Branch,
    pg_version: PgVersion,
    port: u16,
    compute_dir: TempDir,
    binaries_directory: PathBuf,
    child: Option<Child>,
    status: ComputeEndpointStatus,
    channel_binding_signature: Option<Vec<u8>>,
}

impl ComputeEndpoint {
    pub fn new(branch: Branch, pg_version: PgVersion, binaries_directory: PathBuf) -> Result<Self, anyhow::Error> {
        let port = Self::generate_random_port();
        // TODO(matisiekpl): add support for tls sni routing
        let pgdata_dir = TempDir::with_prefix(format!("compute_{}_", branch.timeline_id))?;

        Ok(Self {
            branch,
            pg_version,
            port,
            compute_dir: pgdata_dir,
            binaries_directory,
            child: None,
            status: ComputeEndpointStatus::Stopped,
            channel_binding_signature: None,
        })
    }

    pub fn launch(&mut self) -> Result<(), anyhow::Error> {
        if self.status == ComputeEndpointStatus::Running {
            return Err(anyhow!("Compute endpoint is already running"));
        }
        if self.status == ComputeEndpointStatus::Starting {
            return Err(anyhow!("Compute endpoint is already starting"));
        }
        if self.status == ComputeEndpointStatus::Stopping {
            return Err(anyhow!("Compute endpoint is currently stopping"));
        }

        self.status = ComputeEndpointStatus::Starting;

        self.generate_certificates();
        let config = self.generate_config();
        let config_path = self.compute_dir.path().join("config.json");
        fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;

        let compute_ctl_binary = self.binaries_directory.join("compute_ctl");
        let pgbin = self.binaries_directory.join(format!(
            "pg_install/{}/bin/postgres",
            self.pg_version
        ));

        let connection_string =
            format!("postgresql://cloud_admin@localhost:{}/postgres", self.port);

        let mut cmd = Command::new(&compute_ctl_binary);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            unsafe {
                cmd.pre_exec(|| {
                    #[cfg(target_os = "linux")]
                    libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM, 0, 0, 0);
                    #[cfg(target_os = "macos")]
                    libc::setpgid(0, 0);
                    Ok(())
                });
            }
        }

        let mut child = cmd
            .env_clear()
            .arg("--pgdata")
            .arg(self.compute_dir.path().join("pg_data").to_str().unwrap())
            .arg("--pgbin")
            .arg(pgbin.to_str().unwrap())
            .arg("--compute-id")
            .arg(format!("compute-{}", self.branch.timeline_id))
            .arg("--connstr")
            .arg(&connection_string)
            .arg("--config")
            .arg(config_path.to_str().unwrap())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let pid = child.id();
        let result = wait_for_output_timeout(
            child.stderr.take().unwrap(),
            "listening on IPv4 address",
            true,
            true,
            Some(std::time::Duration::from_secs(50)),
        );

        if let Err(e) = result {
            tracing::error!(
                "Compute endpoint {} failed to start: {}",
                self.branch.timeline_id,
                e
            );
            child.kill().ok();
            child.wait().ok();
            self.status = ComputeEndpointStatus::Failed;
            return Err(anyhow!("Compute endpoint failed to start: {}", e));
        }

        self.child = Some(child);
        self.status = ComputeEndpointStatus::Running;
        tracing::info!(
            "Compute endpoint {} started on PID: {}, port: {}",
            self.branch.timeline_id,
            pid,
            self.port
        );

        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<(), anyhow::Error> {
        if self.status == ComputeEndpointStatus::Stopped {
            return Err(anyhow!("Compute endpoint is already stopped"));
        }
        if self.status == ComputeEndpointStatus::Stopping {
            return Err(anyhow!("Compute endpoint is already stopping"));
        }

        self.status = ComputeEndpointStatus::Stopping;

        if let Some(mut child) = self.child.take() {
            #[cfg(unix)]
            {
                let pid = child.id() as i32;
                tracing::debug!("Sending SIGINT to compute process group: {}", pid);
                unsafe {
                    libc::killpg(pid, libc::SIGINT);
                }
                for _ in 0..50 {
                    match child.try_wait() {
                        Ok(Some(_)) => {
                            self.status = ComputeEndpointStatus::Stopped;
                            tracing::info!("Compute endpoint {} stopped gracefully", self.branch.timeline_id);
                            return Ok(());
                        }
                        Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                        Err(e) => {
                            tracing::warn!("Error waiting for compute endpoint {}: {}", self.branch.timeline_id, e);
                            break;
                        }
                    }
                }
                tracing::warn!("Compute endpoint {} did not stop gracefully, sending SIGKILL", self.branch.timeline_id);
            }
            child.kill().ok();
            child.wait().ok();
        }

        self.status = ComputeEndpointStatus::Stopped;
        tracing::info!("Compute endpoint {} stopped", self.branch.timeline_id);
        Ok(())
    }

    pub fn get_status(&self) -> ComputeEndpointStatus {
        self.status
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    fn generate_certificates(&mut self) {
        let ca_key = KeyPair::generate().expect("failed to generate CA key");
        let mut ca_params = CertificateParams::default();
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
        ca_params.distinguished_name = {
            let mut dn = DistinguishedName::new();
            dn.push(
                rcgen::DnType::CommonName,
                DnValue::PrintableString("neond".try_into().unwrap()),
            );
            dn
        };
        let ca_cert = ca_params
            .self_signed(&ca_key)
            .expect("failed to self-sign CA");

        let issuer = Issuer::new(ca_params, &ca_key);

        fs::write(self.compute_dir.path().join("root_ca.pem"), ca_cert.pem())
            .expect("failed to write root_ca.pem");
        fs::write(
            self.compute_dir.path().join("root_ca.key"),
            ca_key.serialize_pem(),
        )
        .expect("failed to write root_ca.key");

        let server_key = KeyPair::generate().expect("failed to generate server key");
        let mut server_params = CertificateParams::new(vec!["localhost".to_string()])
            .expect("failed to create server params");
        server_params.distinguished_name = {
            let mut dn = DistinguishedName::new();
            dn.push(
                rcgen::DnType::CommonName,
                DnValue::PrintableString("neond-server".try_into().unwrap()),
            );
            dn
        };
        server_params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
        let server_cert = server_params
            .signed_by(&server_key, &issuer)
            .expect("failed to sign server cert");

        fs::write(
            self.compute_dir.path().join("server.pem"),
            server_cert.pem(),
        )
        .expect("failed to write server.pem");
        let server_key_path = self.compute_dir.path().join("server.key");
        fs::write(&server_key_path, server_key.serialize_pem())
            .expect("failed to write server.key");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&server_key_path, fs::Permissions::from_mode(0o600))
                .expect("failed to set server.key permissions");
        }

        let client_key = KeyPair::generate().expect("failed to generate client key");
        let mut client_params = CertificateParams::default();
        client_params.distinguished_name = {
            let mut dn = DistinguishedName::new();
            dn.push(
                rcgen::DnType::CommonName,
                DnValue::PrintableString("neond-client".try_into().unwrap()),
            );
            dn
        };
        client_params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
        client_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];
        let client_cert = client_params
            .signed_by(&client_key, &issuer)
            .expect("failed to sign client cert");

        fs::write(
            self.compute_dir.path().join("client.pem"),
            client_cert.pem(),
        )
        .expect("failed to write client.pem");
        fs::write(
            self.compute_dir.path().join("client.key"),
            client_key.serialize_pem(),
        )
        .expect("failed to write client.key");

        let cert_der = server_cert.der();
        let channel_binding_signature = Sha256::digest(cert_der.as_ref()).to_vec();
        self.channel_binding_signature = Some(channel_binding_signature);
    }

    fn encrypt_password(&self, password: String) -> Result<String, anyhow::Error> {
        let channel_binding = self.channel_binding_signature.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Failed to encrypt password. Channel binding signature is missing.")
        })?;

        const ITERATIONS: u32 = 4096;

        let mut salted_password = [0u8; 32];
        pbkdf2_hmac::<Sha256>(
            password.as_bytes(),
            channel_binding,
            ITERATIONS,
            &mut salted_password,
        );

        let mut client_key_mac = Hmac::<Sha256>::new_from_slice(&salted_password)
            .expect("HMAC can take key of any size");
        client_key_mac.update(b"Client Key");
        let client_key = client_key_mac.finalize().into_bytes();

        let stored_key = Sha256::digest(&client_key);

        let mut server_key_mac = Hmac::<Sha256>::new_from_slice(&salted_password)
            .expect("HMAC can take key of any size");
        server_key_mac.update(b"Server Key");
        let server_key = server_key_mac.finalize().into_bytes();

        let salt_b64 = BASE64_STANDARD.encode(channel_binding);
        let stored_key_b64 = BASE64_STANDARD.encode(&stored_key);
        let server_key_b64 = BASE64_STANDARD.encode(&server_key);

        Ok(format!(
            "SCRAM-SHA-256${ITERATIONS}:{salt_b64}${stored_key_b64}:{server_key_b64}"
        ))
    }

    fn generate_random_port() -> u16 {
        // TODO(matisiekpl): randomize on predefined range
        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
        listener.local_addr().unwrap().port()
    }

    fn setup_pg_conf(&self) -> PostgresConf {
        let mut conf = PostgresConf::new();
        conf.append("max_wal_senders", "10");
        conf.append("wal_log_hints", "off");
        conf.append("max_replication_slots", "10");
        conf.append("hot_standby", "on");
        conf.append("shared_buffers", "1MB");
        conf.append("effective_io_concurrency", "2");
        conf.append("fsync", "off");
        conf.append("max_connections", "100");
        conf.append("wal_level", "logical");
        conf.append("wal_sender_timeout", "5s");
        conf.append("listen_addresses", "127.0.0.1");
        conf.append("port", &self.port.to_string());
        conf.append("wal_keep_size", "0");
        conf.append("restart_after_crash", "off");
        conf.append("shared_preload_libraries", "neon");

        conf.append_line("");
        // Configure backpressure
        conf.append("max_replication_write_lag", "15MB");
        conf.append("max_replication_flush_lag", "10GB");

        // Configure Postgres to connect to the safekeepers
        conf.append("synchronous_standby_names", "walproposer");
        conf.append("neon.safekeepers", "localhost:5454");

        conf.append("password_encryption", "scram-sha-256");

        conf.append_line("");
        // Configure HBA
        self.write_pg_hba();
        conf.append(
            "hba_file",
            self.compute_dir
                .path()
                .join("pg_hba.conf")
                .to_str()
                .unwrap(),
        );

        conf.append_line("");
        // Configure SSL
        conf.append("ssl", "on");
        conf.append(
            "ssl_cert_file",
            self.compute_dir.path().join("server.pem").to_str().unwrap(),
        );
        conf.append(
            "ssl_key_file",
            self.compute_dir.path().join("server.key").to_str().unwrap(),
        );
        conf.append(
            "ssl_ca_file",
            self.compute_dir
                .path()
                .join("root_ca.pem")
                .to_str()
                .unwrap(),
        );

        conf
    }

    fn write_pg_hba(&self) {
        let pg_hba = include_str!("pg_hba.conf");
        fs::write(self.compute_dir.path().join("pg_hba.conf"), pg_hba)
            .expect("failed to write pg_hba.conf");
    }

    fn generate_config(&self) -> ComputeConfig {
        let postgresql_conf = self.setup_pg_conf().to_string();

        let tenant_id = TenantId::from_str(&self.branch.project_id.simple().to_string())
            .expect("Failed to parse tenant_id");
        let timeline_id = TimelineId::from_str(&self.branch.timeline_id.simple().to_string())
            .expect("Failed to parse timeline_id");

        let mut shards = HashMap::new();
        shards.insert(
            ShardIndex::unsharded(),
            PageserverShardInfo {
                pageservers: vec![PageserverShardConnectionInfo {
                    id: Some(NodeId(1)),
                    libpq_url: Some("postgres://cloud_admin@127.0.0.1:64000".to_string()),
                    grpc_url: None,
                }],
            },
        );

        let password = postgres_protocol::password::scram_sha_256(
            String::from(self.branch.password.clone()).as_bytes(),
        );
        let spec = ComputeSpec {
            format_version: 1.0,
            operation_uuid: None,
            features: vec![],
            swap_size_bytes: None,
            disk_quota_bytes: None,
            disable_lfc_resizing: None,
            cluster: Cluster {
                cluster_id: None,
                name: None,
                state: None,
                roles: vec![Role {
                    name: PgIdent::from("postgres"),
                    encrypted_password: Some(password),
                    options: None,
                }],
                databases: vec![Database {
                    name: PgIdent::from("postgres"),
                    owner: PgIdent::from("postgres"),
                    options: None,
                    restrict_conn: false,
                    invalid: false,
                }],
                postgresql_conf: Some(postgresql_conf),
                settings: None,
            },
            delta_operations: None,
            skip_pg_catalog_updates: false,
            tenant_id: Some(tenant_id),
            timeline_id: Some(timeline_id),
            pageserver_connection_info: Some(PageserverConnectionInfo {
                shard_count: ShardCount::unsharded(),
                stripe_size: None,
                shards,
                prefer_protocol: Default::default(),
            }),
            pageserver_connstring: Some("postgres://cloud_admin@127.0.0.1:64000".to_string()),
            shard_stripe_size: None,
            project_id: None,
            branch_id: None,
            endpoint_id: Some(format!("compute-{}", self.branch.timeline_id)),
            safekeepers_generation: None,
            safekeeper_connstrings: vec!["127.0.0.1:5454".to_string()],
            mode: ComputeMode::Primary,
            storage_auth_token: None,
            remote_extensions: None,
            pgbouncer_settings: None,
            local_proxy_config: None,
            reconfigure_concurrency: 1,
            drop_subscriptions_before_start: false,
            audit_log_level: ComputeAudit::Disabled,
            logs_export_host: None,
            endpoint_storage_addr: None,
            endpoint_storage_token: None,
            autoprewarm: false,
            offload_lfc_interval_seconds: None,
            suspend_timeout_seconds: -1,
            databricks_settings: None,
        };

        ComputeConfig {
            spec: Some(spec),
            compute_ctl_config: ComputeCtlConfig::default(),
        }
    }
}

impl Drop for ComputeEndpoint {
    fn drop(&mut self) {
        if self.status == ComputeEndpointStatus::Running
            || self.status == ComputeEndpointStatus::Starting
        {
            if let Err(e) = self.shutdown() {
                tracing::error!("Failed to shutdown compute endpoint: {}", e);
            }
        }
    }
}
