use anyhow::anyhow;
use neon_compute_api::responses::{ComputeConfig, ComputeCtlConfig};
use neon_compute_api::spec::{
    Cluster, ComputeAudit, ComputeMode, ComputeSpec, PageserverConnectionInfo,
    PageserverShardConnectionInfo, PageserverShardInfo,
};
use neon_utils::id::{NodeId, TenantId, TimelineId};
use neon_utils::shard::{ShardCount, ShardIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::str::FromStr;
use tempfile::TempDir;
use uuid::Uuid;

use crate::utils::stdout::wait_for_output;

use crate::mgmt::model::branch::Branch;
use neon_control_plane::postgresql_conf::PostgresConf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputeEndpointStatus {
    Running,
    Stopped,
}

pub struct ComputeEndpoint {
    branch: Branch,
    port: u16,
    pgdata_dir: TempDir,
    binaries_directory: PathBuf,
    child: Option<Child>,
    status: ComputeEndpointStatus,
}

impl ComputeEndpoint {
    pub fn new(branch: Branch, binaries_directory: PathBuf) -> Result<Self, anyhow::Error> {
        let port = Self::generate_random_port();
        let pgdata_dir = TempDir::with_prefix(format!("compute_{}_", branch.timeline_id))?;

        Ok(Self {
            branch,
            port,
            pgdata_dir,
            binaries_directory,
            child: None,
            status: ComputeEndpointStatus::Stopped,
        })
    }

    pub fn launch(&mut self) -> Result<(), anyhow::Error> {
        // TODO(matisiekpl): set password
        // TODO(matisiekpl): set tls
        if self.status == ComputeEndpointStatus::Running {
            return Err(anyhow!("Compute endpoint is already running"));
        }

        let config = self.generate_config();
        let config_path = self.pgdata_dir.path().join("config.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;

        let compute_ctl_binary = self.binaries_directory.join("compute_ctl");
        let pgbin = self.binaries_directory.join(format!(
            "pg_install/{}/bin/postgres",
            self.branch.pg_version
        ));

        let connection_string = format!(
            "postgresql://neon_superuser@localhost:{}/postgres",
            self.port
        );

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
            .arg(self.pgdata_dir.path().to_str().unwrap())
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
        wait_for_output(
            child.stderr.take().unwrap(),
            "listening on IPv4 address",
            true,
            true,
        )?;

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

        if let Some(mut child) = self.child.take() {
            #[cfg(unix)]
            unsafe {
                tracing::debug!("Sending SIGINT to compute process: {}", child.id());
                libc::killpg(child.id() as i32, libc::SIGINT);
                std::thread::sleep(std::time::Duration::from_secs(5));
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

        conf
    }

    fn generate_random_port() -> u16 {
        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
        listener.local_addr().unwrap().port()
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
                    libpq_url: Some("postgres://no_user@127.0.0.1:64000".to_string()),
                    grpc_url: None,
                }],
            },
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
                roles: vec![],
                databases: vec![],
                postgresql_conf: Some(postgresql_conf),
                settings: None,
            },
            delta_operations: None,
            skip_pg_catalog_updates: true,
            tenant_id: Some(tenant_id),
            timeline_id: Some(timeline_id),
            pageserver_connection_info: Some(PageserverConnectionInfo {
                shard_count: ShardCount::unsharded(),
                stripe_size: None,
                shards,
                prefer_protocol: Default::default(),
            }),
            pageserver_connstring: Some("postgres://neon_superuser@127.0.0.1:64000".to_string()),
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
        if self.status == ComputeEndpointStatus::Running {
            if let Err(e) = self.shutdown() {
                tracing::error!("Failed to shutdown compute endpoint: {}", e);
            }
        }
    }
}
