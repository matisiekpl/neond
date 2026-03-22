mod death;
mod pageserver;
mod postgres;
mod process;
mod stdout;
mod tracer;

use crate::daemon::process::ProcessControl;
use std::path::PathBuf;
use tracing::info;

pub struct Daemon {
    daemon_directory: PathBuf,
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
    pub fn new(daemon_directory: PathBuf) -> Self {
        let verbose = cfg!(debug_assertions);

        let pageserver_working_directory = daemon_directory.join("pageserver");
        let safekeeper_working_directory = daemon_directory.join("safekeeper");

        let storage_controller_postgres = postgres::Postgres::new(
            "storage_controller_db",
            daemon_directory.clone(),
            "storage_controller_pg_data",
            5431,
            // TODO(matisiekpl): change password
            "mateuszek".to_string(),
        );
        let management_postgres = postgres::Postgres::new(
            "management_db",
            daemon_directory.clone(),
            "management_pg_data",
            5430,
            // TODO(matisiekpl): change password
            "mateuszek".to_string(),
        );

        let storage_broker = ProcessControl::new(
            "Storage broker",
            daemon_directory.join("binaries/storage_broker"),
            ["-l", "127.0.0.1:50051"],
            daemon_directory.clone(),
            "listening",
            verbose,
        );

        let storage_controller = ProcessControl::new(
            "Storage controller",
            daemon_directory.join("binaries/storage_controller"),
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
            daemon_directory.clone(),
            "Serving HTTP on 127.0.0.1:1234",
            verbose,
        );

        let safekeeper = ProcessControl::new(
            "Safekeeper",
            daemon_directory.join("binaries/safekeeper"),
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
            daemon_directory.clone(),
            "starting safekeeper WAL service on",
            verbose,
        );

        let pageserver = ProcessControl::new(
            "Pageserver",
            daemon_directory.join("binaries/pageserver"),
            [
                "-D",
                pageserver_working_directory
                    .to_str()
                    .unwrap()
                    .to_owned()
                    .as_str(),
            ],
            daemon_directory.clone(),
            "Starting pageserver http handler on 127.0.0.1:9898",
            verbose,
        );

        Daemon {
            daemon_directory: daemon_directory.clone(),
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
        pageserver::write_pageserver_init_files(&self.daemon_directory)?;
        self.pageserver.start()?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Stopping daemon...");
        self.pageserver.stop()?;
        self.safekeeper.stop()?;
        self.tracer.stop();
        self.storage_broker.stop()?;
        self.storage_controller.stop()?;
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
