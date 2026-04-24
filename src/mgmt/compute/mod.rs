use base64::prelude::*;
use hmac::Hmac;
use hmac::Mac;
use neon_compute_api::responses::{ComputeConfig, ComputeCtlConfig};
use neon_compute_api::spec::{
    Cluster, ComputeAudit, ComputeMode, ComputeSpec, Database, PageserverConnectionInfo,
    PageserverShardConnectionInfo, PageserverShardInfo, PgIdent, Role,
};
use neon_utils::auth::Scope;
use neon_utils::id::{NodeId, TenantId, TimelineId};
use neon_utils::shard::{ShardCount, ShardIndex};
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

use crate::mgmt::dto::config::Config;
use crate::mgmt::dto::error::{AppError, Result};
use base64::{Engine as _, engine::general_purpose};
use rand::{Rng, RngCore};

fn path_str<'path>(path: &'path std::path::Path) -> Result<&'path str> {
    path.to_str()
        .ok_or_else(|| AppError::ComputeProcessStartupFailed {
            reason: format!("path contains non-UTF-8 characters: {}", path.display()),
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::mgmt::schema::schema::sql_types::ComputeEndpointStatus"]
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
    port: Option<u16>,
    preferred_port: Option<u16>,
    metrics_port: Option<u16>,
    pid: Option<u32>,
    compute_dir: TempDir,
    child: Option<Child>,
    status: ComputeEndpointStatus,
    channel_binding_signature: Option<Vec<u8>>,
    config: Config,
}

#[derive(Clone)]
pub struct ComputeEndpointInfo {
    pub(crate) status: ComputeEndpointStatus,
    pub(crate) port: u16,
}

impl ComputeEndpoint {
    pub fn new(
        config: Config,
        branch: Branch,
        pg_version: PgVersion,
        preferred_port: Option<u16>,
    ) -> Result<Self> {
        let pgdata_dir = TempDir::with_prefix(format!("compute_{}_", branch.timeline_id))
            .map_err(|error| AppError::ComputeStartupFailed {
                reason: error.to_string(),
            })?;

        Ok(Self {
            config,
            branch,
            pg_version,
            port: None,
            preferred_port,
            metrics_port: None,
            pid: None,
            compute_dir: pgdata_dir,
            child: None,
            status: ComputeEndpointStatus::Stopped,
            channel_binding_signature: None,
        })
    }

    pub fn get_branch(&self) -> &Branch {
        &self.branch
    }

    pub fn launch(&mut self) -> Result<()> {
        if self.status == ComputeEndpointStatus::Running {
            return Err(AppError::ComputeStartupFailed {
                reason: "Compute endpoint is already running".to_string(),
            });
        }
        if self.status == ComputeEndpointStatus::Starting {
            return Err(AppError::ComputeStartupFailed {
                reason: "Compute endpoint is already starting".to_string(),
            });
        }
        if self.status == ComputeEndpointStatus::Stopping {
            return Err(AppError::ComputeStartupFailed {
                reason: "Compute endpoint is currently stopping".to_string(),
            });
        }

        let port = self.resolve_port()?;
        self.port = Some(port);

        self.status = ComputeEndpointStatus::Starting;

        self.generate_certificates()?;
        let config = self.generate_config()?;
        let config_path = self.compute_dir.path().join("config.json");
        let config_json = serde_json::to_string_pretty(&config).map_err(|error| {
            AppError::ComputeProcessStartupFailed {
                reason: error.to_string(),
            }
        })?;
        fs::write(&config_path, config_json).map_err(|error| {
            AppError::ComputeProcessStartupFailed {
                reason: error.to_string(),
            }
        })?;

        let compute_ctl_binary = self.config.neon_binaries_directory.join("compute_ctl");
        let pgbin = self
            .config
            .pg_install_directory
            .join(format!("{}/bin/postgres", self.pg_version));

        let connection_string = format!("postgresql://cloud_admin@localhost:{}/postgres", port);

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

        let metrics_port =
            crate::utils::ports::allocate_random_port().map_err(|error| {
                AppError::ComputeProcessStartupFailed {
                    reason: format!("failed to allocate metrics port: {}", error),
                }
            })?;
        self.metrics_port = Some(metrics_port);

        let pg_data_path = self.compute_dir.path().join("pg_data");
        let mut child = cmd
            .env_clear()
            .arg("--pgdata")
            .arg(path_str(&pg_data_path)?)
            .arg("--pgbin")
            .arg(path_str(&pgbin)?)
            .arg("--compute-id")
            .arg(format!("compute-{}", self.branch.timeline_id))
            .arg("--connstr")
            .arg(&connection_string)
            .arg("--config")
            .arg(path_str(&config_path)?)
            .arg("--external-http-port")
            .arg(metrics_port.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| AppError::ComputeProcessStartupFailed {
                reason: error.to_string(),
            })?;

        let pid = child.id();
        self.pid = Some(pid);
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| AppError::ComputeProcessStartupFailed {
                reason: "stderr was not piped".to_string(),
            })?;
        let result = wait_for_output_timeout(
            stderr,
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
            self.pid = None;
            self.metrics_port = None;
            return Err(AppError::ComputeProcessStartupFailed {
                reason: e.to_string(),
            });
        }

        self.child = Some(child);
        self.status = ComputeEndpointStatus::Running;
        tracing::info!(
            "Compute endpoint {} started on PID: {}, port: {}",
            self.branch.timeline_id,
            pid,
            port,
        );

        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<()> {
        if self.status == ComputeEndpointStatus::Stopped {
            return Err(AppError::ComputeShutdownFailed {
                reason: "Compute endpoint is already stopped".to_string(),
            });
        }
        if self.status == ComputeEndpointStatus::Stopping {
            return Err(AppError::ComputeShutdownFailed {
                reason: "Compute endpoint is already stopping".to_string(),
            });
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
                            tracing::info!(
                                "Compute endpoint {} stopped gracefully",
                                self.branch.timeline_id
                            );
                            return Ok(());
                        }
                        Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                        Err(e) => {
                            tracing::warn!(
                                "Error waiting for compute endpoint {}: {}",
                                self.branch.timeline_id,
                                e
                            );
                            break;
                        }
                    }
                }
                tracing::warn!(
                    "Compute endpoint {} did not stop gracefully, sending SIGKILL",
                    self.branch.timeline_id
                );
            }
            child.kill().ok();
            child.wait().ok();
        }

        self.status = ComputeEndpointStatus::Stopped;
        self.pid = None;
        self.metrics_port = None;
        tracing::info!("Compute endpoint {} stopped", self.branch.timeline_id);
        Ok(())
    }

    pub fn get_status(&self) -> ComputeEndpointStatus {
        self.status
    }

    pub fn get_port(&self) -> u16 {
        self.port.unwrap_or(0)
    }

    pub fn get_pid(&self) -> Option<u32> {
        self.pid
    }

    pub fn get_metrics_port(&self) -> Option<u16> {
        self.metrics_port
    }

    fn generate_certificates(&mut self) -> Result<()> {
        let cert_error = |component: &str, reason: String| AppError::ComputeCertificateGenerationFailed {
            component: component.to_string(),
            reason,
        };
        let write_cert_file = |path: std::path::PathBuf, contents: String, component: &str| -> Result<()> {
            fs::write(path, contents).map_err(|error| cert_error(component, error.to_string()))
        };
        let printable = |value: &str, component: &str| -> Result<DnValue> {
            value
                .try_into()
                .map(DnValue::PrintableString)
                .map_err(|error: rcgen::Error| cert_error(component, error.to_string()))
        };

        let ca_key = KeyPair::generate().map_err(|error| cert_error("ca_key", error.to_string()))?;
        let mut ca_params = CertificateParams::default();
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        ca_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
        ca_params.distinguished_name = {
            let mut dn = DistinguishedName::new();
            dn.push(rcgen::DnType::CommonName, printable("neond", "ca_cert")?);
            dn
        };
        let ca_cert = ca_params
            .self_signed(&ca_key)
            .map_err(|error| cert_error("ca_cert", error.to_string()))?;

        let issuer = Issuer::new(ca_params, &ca_key);

        write_cert_file(
            self.compute_dir.path().join("root_ca.pem"),
            ca_cert.pem(),
            "root_ca.pem",
        )?;
        write_cert_file(
            self.compute_dir.path().join("root_ca.key"),
            ca_key.serialize_pem(),
            "root_ca.key",
        )?;

        let server_key =
            KeyPair::generate().map_err(|error| cert_error("server_key", error.to_string()))?;
        let mut server_params = CertificateParams::new(vec!["localhost".to_string()])
            .map_err(|error| cert_error("server_params", error.to_string()))?;
        server_params.distinguished_name = {
            let mut dn = DistinguishedName::new();
            dn.push(
                rcgen::DnType::CommonName,
                printable("neond-server", "server_cert")?,
            );
            dn
        };
        server_params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
        let server_cert = server_params
            .signed_by(&server_key, &issuer)
            .map_err(|error| cert_error("server_cert", error.to_string()))?;

        write_cert_file(
            self.compute_dir.path().join("server.pem"),
            server_cert.pem(),
            "server.pem",
        )?;
        let server_key_path = self.compute_dir.path().join("server.key");
        write_cert_file(
            server_key_path.clone(),
            server_key.serialize_pem(),
            "server.key",
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&server_key_path, fs::Permissions::from_mode(0o600))
                .map_err(|error| cert_error("server.key_permissions", error.to_string()))?;
        }

        let client_key =
            KeyPair::generate().map_err(|error| cert_error("client_key", error.to_string()))?;
        let mut client_params = CertificateParams::default();
        client_params.distinguished_name = {
            let mut dn = DistinguishedName::new();
            dn.push(
                rcgen::DnType::CommonName,
                printable("neond-client", "client_cert")?,
            );
            dn
        };
        client_params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
        client_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];
        let client_cert = client_params
            .signed_by(&client_key, &issuer)
            .map_err(|error| cert_error("client_cert", error.to_string()))?;

        write_cert_file(
            self.compute_dir.path().join("client.pem"),
            client_cert.pem(),
            "client.pem",
        )?;
        write_cert_file(
            self.compute_dir.path().join("client.key"),
            client_key.serialize_pem(),
            "client.key",
        )?;

        let cert_der = server_cert.der();
        let channel_binding_signature = Sha256::digest(cert_der.as_ref()).to_vec();
        self.channel_binding_signature = Some(channel_binding_signature);
        Ok(())
    }

    fn resolve_port(&self) -> Result<u16> {
        if let Some(preferred) = self.preferred_port {
            if std::net::TcpListener::bind(("127.0.0.1", preferred)).is_ok() {
                return Ok(preferred);
            }
            tracing::warn!(
                "Preferred port {} for branch {} is unavailable, falling back to random port",
                preferred,
                self.branch.timeline_id
            );
        }
        self.generate_random_port()
    }

    fn generate_random_port(&self) -> Result<u16> {
        const MAX_ATTEMPTS: usize = 100;
        let mut rng = rand::rng();
        for _ in 0..MAX_ATTEMPTS {
            let port = rng.random_range(self.config.port_range.0..=self.config.port_range.1);

            if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
                return Ok(port);
            }
        }
        Err(AppError::ComputePortAllocationFailed)
    }

    fn setup_pg_conf(&self) -> Result<PostgresConf> {
        let mut conf = PostgresConf::new();

        conf.append("max_wal_senders", "10");
        conf.append("wal_log_hints", "off");
        conf.append("max_replication_slots", "10");
        conf.append("hot_standby", "on");

        conf.append("shared_buffers", "128MB");
        conf.append("effective_cache_size", "512MB");
        conf.append("work_mem", "8MB");
        conf.append("maintenance_work_mem", "128MB");
        conf.append("max_connections", "100");

        conf.append("effective_io_concurrency", "100");
        conf.append("random_page_cost", "1.1");
        conf.append("fsync", "off");
        conf.append("synchronous_commit", "on");

        conf.append("wal_level", "logical");
        conf.append("wal_sender_timeout", "60s");
        conf.append("wal_keep_size", "0");
        conf.append("restart_after_crash", "off");

        conf.append("listen_addresses", "0.0.0.0");
        conf.append("port", &self.port.unwrap_or(0).to_string());
        conf.append("shared_preload_libraries", "neon");

        conf.append("jit", "off");

        conf.append("statement_timeout", "0");
        conf.append("idle_in_transaction_session_timeout", "600000");

        conf.append("autovacuum_max_workers", "4");
        conf.append("autovacuum_naptime", "10s");
        conf.append("autovacuum_vacuum_scale_factor", "0.05");
        conf.append("autovacuum_analyze_scale_factor", "0.02");
        conf.append("autovacuum_vacuum_cost_limit", "2000");

        conf.append("log_min_duration_statement", "1000");
        conf.append("log_connections", "on");
        conf.append("log_disconnections", "on");
        conf.append("log_checkpoints", "on");
        conf.append("log_lock_waits", "on");
        conf.append("log_temp_files", "0");
        conf.append("log_autovacuum_min_duration", "1000");
        conf.append("log_line_prefix", "'%m [%p] %q%u@%d '");

        conf.append_line("");
        conf.append("max_replication_write_lag", "500MB");
        conf.append("max_replication_flush_lag", "10GB");

        conf.append("synchronous_standby_names", "walproposer");
        conf.append("neon.safekeepers", "localhost:5454");

        conf.append("password_encryption", "scram-sha-256");

        conf.append_line("");
        self.write_pg_hba()?;
        let hba_path = self.compute_dir.path().join("pg_hba.conf");
        conf.append("hba_file", path_str(&hba_path)?);

        conf.append_line("");
        conf.append("ssl", "on");
        let server_pem_path = self.compute_dir.path().join("server.pem");
        conf.append("ssl_cert_file", path_str(&server_pem_path)?);
        let server_key_path = self.compute_dir.path().join("server.key");
        conf.append("ssl_key_file", path_str(&server_key_path)?);
        let root_ca_path = self.compute_dir.path().join("root_ca.pem");
        conf.append("ssl_ca_file", path_str(&root_ca_path)?);

        Ok(conf)
    }

    fn write_pg_hba(&self) -> Result<()> {
        let pg_hba = include_str!("pg_hba.conf");
        fs::write(self.compute_dir.path().join("pg_hba.conf"), pg_hba).map_err(|error| {
            AppError::ComputeProcessStartupFailed {
                reason: format!("failed to write pg_hba.conf: {}", error),
            }
        })
    }

    fn generate_config(&self) -> Result<ComputeConfig> {
        let postgresql_conf = self.setup_pg_conf()?.to_string();

        let tenant_id_value = self.branch.project_id.simple().to_string();
        let tenant_id = TenantId::from_str(&tenant_id_value).map_err(|_| {
            AppError::TenantIdInvalid {
                value: tenant_id_value,
            }
        })?;
        let timeline_id_value = self.branch.timeline_id.simple().to_string();
        let timeline_id = TimelineId::from_str(&timeline_id_value).map_err(|_| {
            AppError::TimelineIdInvalid {
                value: timeline_id_value,
            }
        })?;

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
            storage_auth_token: Some(
                self.config
                    .component_auth
                    .generate_token(Scope::Tenant, Some(tenant_id))?,
            ),
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

        Ok(ComputeConfig {
            spec: Some(spec),
            compute_ctl_config: ComputeCtlConfig::default(),
        })
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
