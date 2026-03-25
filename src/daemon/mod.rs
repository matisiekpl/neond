mod pageserver;
mod postgres;
mod process;
mod tracer;

use crate::daemon::process::ProcessControl;
use crate::mgmt::dto::config::Config;
use std::path::PathBuf;
use tracing::info;

pub struct Daemon {
    config: Config,
    storage_controller_postgres: postgres::Postgres,
    management_postgres: postgres::Postgres,
    tracer: tracer::Tracer,

    pageserver_working_directory: PathBuf,
    safekeeper_working_directory: PathBuf,

    storage_broker: ProcessControl,
    storage_controller: ProcessControl,
    pageserver: ProcessControl,
    safekeeper: ProcessControl,
}

impl Daemon {
    pub fn new(config: Config) -> Self {
        let verbose = cfg!(debug_assertions);

        let pageserver_working_directory = config.daemon_directory.join("pageserver");
        let safekeeper_working_directory = config.daemon_directory.join("safekeeper");

        let storage_controller_postgres = postgres::Postgres::new(
            "storage_controller_db",
            config.daemon_directory.clone(),
            config.binaries_directory.clone(),
            "storage_controller_pg_data",
            5431,
            // TODO(matisiekpl): change password
            "mateuszek".to_string(),
        );
        let management_postgres = postgres::Postgres::new(
            "management_db",
            config.daemon_directory.clone(),
            config.binaries_directory.clone(),
            "management_pg_data",
            5430,
            // TODO(matisiekpl): change password
            "mateuszek".to_string(),
        );

        let storage_broker = ProcessControl::new(
            "Storage broker",
            config.binaries_directory.join("storage_broker"),
            ["-l", "127.0.0.1:50051"],
            config.daemon_directory.clone(),
            "listening",
            verbose,
        );

        let storage_controller = ProcessControl::new(
            "Storage controller",
            config.binaries_directory.join("storage_controller"),
            [
                "-l",
                "127.0.0.1:1234",
                "--database-url",
                storage_controller_postgres.get_connection_uri().as_str(),
                "--dev",
                "--timeline-safekeeper-count",
                "1",
                "--timelines-onto-safekeepers",
                "--control-plane-url",
                "http://127.0.0.1:1235",
            ],
            config.daemon_directory.clone(),
            "Serving HTTP on 127.0.0.1:1234",
            verbose,
        );

        let safekeeper = ProcessControl::new(
            "Safekeeper",
            config.binaries_directory.join("safekeeper"),
            [
                "-D",
                safekeeper_working_directory
                    .to_str()
                    .unwrap()
                    .to_owned()
                    .as_str(),
                "--id",
                "1",
                "--broker-endpoint",
                "http://127.0.0.1:50051",
                "--listen-pg",
                "127.0.0.1:5454",
                "--listen-http",
                "127.0.0.1:7676",
                "--availability-zone",
                "neond-1",
            ],
            config.daemon_directory.clone(),
            "starting safekeeper WAL service on",
            verbose,
        );

        let pageserver = ProcessControl::new(
            "Pageserver",
            config.binaries_directory.join("pageserver"),
            [
                "-D",
                pageserver_working_directory
                    .to_str()
                    .unwrap()
                    .to_owned()
                    .as_str(),
            ],
            config.daemon_directory.clone(),
            "Starting pageserver http handler on 127.0.0.1:9898",
            verbose,
        );

        Daemon {
            config,
            storage_controller_postgres,
            management_postgres,
            tracer: tracer::Tracer::new(),
            pageserver_working_directory,
            safekeeper_working_directory,
            storage_broker,
            storage_controller,
            pageserver,
            safekeeper,
        }
    }

    pub async fn start(&mut self) -> Result<(), anyhow::Error> {
        self.storage_controller_postgres.init()?;
        self.management_postgres.init()?;
        self.storage_controller_postgres.start()?;
        self.management_postgres.start()?;
        self.tracer.start();
        self.storage_broker.start()?;
        self.storage_controller.start()?;
        std::fs::create_dir_all(&self.safekeeper_working_directory)?;
        self.safekeeper.start()?;
        self.register_safekeeper().await?;
        std::fs::create_dir_all(&self.pageserver_working_directory)?;
        pageserver::write_pageserver_init_files(
            &self.config.daemon_directory,
            &self.config.binaries_directory,
            &self.config.remote_storage_config,
        )?;
        self.pageserver.start()?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Stopping daemon...");
        self.pageserver.stop()?;
        self.safekeeper.stop()?;
        self.storage_controller.stop()?;
        self.storage_broker.stop()?;
        self.tracer.stop();
        self.storage_controller_postgres.stop()?;
        self.management_postgres.stop()?;
        Ok(())
    }

    async fn register_safekeeper(&mut self) -> Result<(), anyhow::Error> {
        let safekeeper_http_client = reqwest::Client::new();
        let now = chrono::Utc::now().to_rfc3339();
        let body = serde_json::json!({
            "id": 1,
            "region_id": "neond-1",
            "host": "127.0.0.1",
            "port": 5454,
            "http_port": 7676,
            "version": 1,
            "availability_zone_id": "neond-1",
            "created_at": now,
            "updated_at": now
        });

        let response = safekeeper_http_client
            .post("http://127.0.0.1:1234/control/v1/safekeeper/1")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;
        info!(
            "Registered safekeeper with status code: {:?}",
            response.status().as_u16()
        );

        Ok(())
    }

    pub fn get_management_postgres_uri(&self) -> String {
        self.management_postgres.get_connection_uri()
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            tracing::error!("Failed to stop daemon: {}", e);
        }
    }
}
