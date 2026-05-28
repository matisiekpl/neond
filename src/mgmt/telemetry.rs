use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use posthog_rs::{Client, EU_INGESTION_ENDPOINT, Event, client};
use tokio::time::sleep;
use uuid::Uuid;

use crate::mgmt::service::Services;

const POSTHOG_PROJECT_TOKEN: &str = "phc_Hj5fz06S3SSitBrMSk7QfAK6e2qk3qVrgLVd3TspXXR";
const LIB_NAME: &str = "neond-server";
const LIB_VERSION: &str = env!("CARGO_PKG_VERSION");
const HEARTBEAT_INITIAL_DELAY: Duration = Duration::from_secs(5 * 60);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(60 * 60);
const INSTALLATION_ID_FILE: &str = "installation_id";

pub struct Telemetry {
    installation_id: String,
    started_at: Instant,
    client: Client,
}

impl Telemetry {
    pub async fn new(daemon_directory: &Path) -> Option<Arc<Self>> {
        if is_disabled() {
            tracing::info!("Telemetry disabled via DO_NOT_TRACK or TELEMETRY_DISABLED");
            return None;
        }

        let installation_id = load_or_create_installation_id(daemon_directory).await;
        let client = client((POSTHOG_PROJECT_TOKEN, EU_INGESTION_ENDPOINT)).await;
        let telemetry = Arc::new(Self {
            installation_id,
            started_at: Instant::now(),
            client,
        });

        tracing::info!(
            "Telemetry enabled — anonymous installation_id={}",
            telemetry.installation_id
        );

        Some(telemetry)
    }

    pub fn spawn_heartbeat(self: Arc<Self>, services: Arc<Services>) {
        let telemetry = Arc::clone(&self);
        tokio::spawn(async move {
            telemetry.capture_app_started().await;
            sleep(HEARTBEAT_INITIAL_DELAY).await;
            loop {
                telemetry.send_heartbeat(Arc::clone(&services)).await;
                sleep(HEARTBEAT_INTERVAL).await;
            }
        });
    }

    pub async fn capture_pageview(&self, user_id: Uuid) {
        let mut event = Event::new("$pageview", &user_id.to_string());
        insert_or_log(&mut event, "installation_id", &self.installation_id);
        self.fill_common_properties(&mut event);
        self.send(event).await;
    }

    async fn capture_app_started(&self) {
        let mut event = Event::new("app_started", &self.installation_id);
        self.fill_common_properties(&mut event);
        self.send(event).await;
    }

    async fn send_heartbeat(&self, services: Arc<Services>) {
        let uptime_seconds = self.started_at.elapsed().as_secs();
        let projects_count = match services.project().count_all().await {
            Ok(count) => Some(count),
            Err(error) => {
                tracing::debug!("Telemetry: failed to count projects: {}", error);
                None
            }
        };
        let branches_count = match services.branch().count_all().await {
            Ok(count) => Some(count),
            Err(error) => {
                tracing::debug!("Telemetry: failed to count branches: {}", error);
                None
            }
        };
        let endpoints_running_count = services.endpoint().count_running().await;

        let mut event = Event::new("heartbeat", &self.installation_id);
        self.fill_common_properties(&mut event);
        insert_or_log(&mut event, "uptime_seconds", uptime_seconds);
        insert_or_log(&mut event, "endpoints_running_count", endpoints_running_count);
        if let Some(value) = projects_count {
            insert_or_log(&mut event, "projects_count", value);
        }
        if let Some(value) = branches_count {
            insert_or_log(&mut event, "branches_count", value);
        }
        self.send(event).await;
    }

    fn fill_common_properties(&self, event: &mut Event) {
        insert_or_log(event, "$lib", LIB_NAME);
        insert_or_log(event, "$lib_version", LIB_VERSION);
        insert_or_log(event, "os", std::env::consts::OS);
        insert_or_log(event, "arch", std::env::consts::ARCH);
    }

    async fn send(&self, event: Event) {
        if let Err(error) = self.client.capture(event).await {
            tracing::debug!("Telemetry: failed to capture event: {}", error);
        }
    }
}

fn insert_or_log<T: serde::Serialize>(event: &mut Event, key: &str, value: T) {
    if let Err(error) = event.insert_prop(key, value) {
        tracing::debug!("Telemetry: failed to insert property {}: {}", key, error);
    }
}

fn is_disabled() -> bool {
    matches!(
        std::env::var("DO_NOT_TRACK").as_deref(),
        Ok("1") | Ok("true")
    ) || matches!(
        std::env::var("TELEMETRY_DISABLED").as_deref(),
        Ok("1") | Ok("true")
    )
}

async fn load_or_create_installation_id(daemon_directory: &Path) -> String {
    let path: PathBuf = daemon_directory.join(INSTALLATION_ID_FILE);

    if let Ok(content) = tokio::fs::read_to_string(&path).await {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    let new_id = Uuid::new_v4().to_string();
    if let Err(error) = tokio::fs::create_dir_all(daemon_directory).await {
        tracing::debug!(
            "Telemetry: failed to create daemon directory for installation_id: {}",
            error
        );
    }
    if let Err(error) = tokio::fs::write(&path, &new_id).await {
        tracing::debug!(
            "Telemetry: failed to persist installation_id to {}: {}",
            path.display(),
            error
        );
    }
    new_id
}
