use std::process::Command;
use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use tokio::sync::{Mutex, MutexGuard, OnceCell};
use tokio_postgres::NoTls;

pub const BASE_URL: &str = "http://127.0.0.1:3100/api";
const COMPOSE_FILE: &str = "docker/compose.e2e.yaml";
const IMAGE: &str = "neond/neond:test";

static DAEMON: OnceCell<()> = OnceCell::const_new();
static SERIAL: Mutex<()> = Mutex::const_new(());

pub async fn exclusive() -> MutexGuard<'static, ()> {
    SERIAL.lock().await
}

fn compose(arguments: &[&str]) -> std::process::Output {
    Command::new("docker")
        .arg("compose")
        .arg("-f")
        .arg(COMPOSE_FILE)
        .args(arguments)
        .output()
        .expect("failed to run docker compose")
}

fn ensure_image() {
    let output = Command::new("docker")
        .args(["image", "inspect", IMAGE])
        .output()
        .expect("failed to run docker image inspect");
    assert!(
        output.status.success(),
        "docker image {IMAGE} is missing. Build it first:\n    docker build -t {IMAGE} ."
    );
}

async fn start_daemon() {
    ensure_image();
    compose(&["down", "-v"]);

    let output = compose(&["up", "-d", "--wait"]);
    assert!(
        output.status.success(),
        "docker compose up failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    println!("[e2e] neond is up at {BASE_URL}");
    println!("[e2e] logs: docker compose -f {COMPOSE_FILE} logs neond");
    println!("[e2e] teardown: docker compose -f {COMPOSE_FILE} down -v");
}

pub async fn api() -> Api {
    DAEMON.get_or_init(start_daemon).await;
    Api::authenticated().await
}

pub struct Api {
    client: Client,
    token: String,
}

impl Api {
    async fn authenticated() -> Self {
        let client = Client::new();
        let register = client
            .post(format!("{BASE_URL}/auth/register"))
            .json(&json!({
                "name": "E2E User",
                "email": "e2e@example.com",
                "password": "e2e_password_123"
            }))
            .send()
            .await
            .expect("failed to send register request");

        let token = if register.status().is_success() {
            register.json::<Value>().await.expect("bad register body")["token"]
                .as_str()
                .expect("token missing")
                .to_string()
        } else {
            let login = client
                .post(format!("{BASE_URL}/auth/login"))
                .json(&json!({
                    "email": "e2e@example.com",
                    "password": "e2e_password_123"
                }))
                .send()
                .await
                .expect("failed to send login request");
            assert!(login.status().is_success(), "login failed: {}", login.status());
            login.json::<Value>().await.expect("bad login body")["token"]
                .as_str()
                .expect("token missing")
                .to_string()
        };

        Api { client, token }
    }

    async fn send(&self, request: reqwest::RequestBuilder) -> (StatusCode, Value) {
        let response = request
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .expect("request failed");
        let status = response.status();
        let body = response.json::<Value>().await.unwrap_or(Value::Null);
        (status, body)
    }

    async fn expect_ok(&self, request: reqwest::RequestBuilder) -> Value {
        let (status, body) = self.send(request).await;
        assert!(status.is_success(), "request failed with {status}: {body}");
        body
    }

    pub async fn create_organization(&self, name: &str) -> String {
        let body = self
            .expect_ok(
                self.client
                    .post(format!("{BASE_URL}/organizations"))
                    .json(&json!({ "name": name })),
            )
            .await;
        body["id"].as_str().expect("organization id missing").to_string()
    }

    pub async fn create_project(&self, organization_id: &str, name: &str) -> String {
        let deadline = std::time::Instant::now() + Duration::from_secs(60);
        loop {
            let (status, body) = self
                .send(
                    self.client
                        .post(format!("{BASE_URL}/organizations/{organization_id}/projects"))
                        .json(&json!({ "name": name })),
                )
                .await;
            if status.is_success() {
                return body["id"].as_str().expect("project id missing").to_string();
            }
            assert!(
                std::time::Instant::now() < deadline,
                "project creation failed with {status}: {body}"
            );
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    pub async fn create_branch(
        &self,
        organization_id: &str,
        project_id: &str,
        name: &str,
        parent_branch_id: Option<&str>,
    ) -> Value {
        self.expect_ok(
            self.client
                .post(format!(
                    "{BASE_URL}/organizations/{organization_id}/projects/{project_id}/branches"
                ))
                .json(&json!({ "name": name, "parent_branch_id": parent_branch_id })),
        )
        .await
    }

    pub async fn get_branch(&self, organization_id: &str, project_id: &str, branch_id: &str) -> Value {
        let branches = self
            .expect_ok(self.client.get(format!(
                "{BASE_URL}/organizations/{organization_id}/projects/{project_id}/branches"
            )))
            .await;
        branches
            .as_array()
            .expect("branch list is not an array")
            .iter()
            .find(|branch| branch["id"].as_str() == Some(branch_id))
            .unwrap_or_else(|| panic!("branch {branch_id} not found"))
            .clone()
    }

    pub async fn start_endpoint(&self, organization_id: &str, project_id: &str, branch_id: &str) -> Value {
        self.expect_ok(self.client.post(format!(
            "{BASE_URL}/organizations/{organization_id}/projects/{project_id}/branches/{branch_id}/endpoint"
        )))
        .await
    }

    pub async fn stop_endpoint(&self, organization_id: &str, project_id: &str, branch_id: &str) {
        self.expect_ok(self.client.delete(format!(
            "{BASE_URL}/organizations/{organization_id}/projects/{project_id}/branches/{branch_id}/endpoint"
        )))
        .await;
    }

    pub async fn endpoint(&self, organization_id: &str, project_id: &str, branch_id: &str) -> Value {
        self.expect_ok(self.client.get(format!(
            "{BASE_URL}/organizations/{organization_id}/projects/{project_id}/branches/{branch_id}/endpoint"
        )))
        .await
    }

    pub async fn endpoint_status(&self, organization_id: &str, project_id: &str, branch_id: &str) -> String {
        let body = self.endpoint(organization_id, project_id, branch_id).await;
        body["status"].as_str().unwrap_or("UNKNOWN").to_uppercase()
    }

    pub async fn lsn_at(
        &self,
        organization_id: &str,
        project_id: &str,
        branch_id: &str,
        timestamp: DateTime<Utc>,
    ) -> String {
        let body = self
            .expect_ok(
                self.client
                    .get(format!(
                        "{BASE_URL}/organizations/{organization_id}/projects/{project_id}/branches/{branch_id}/lsn"
                    ))
                    .query(&[("timestamp", timestamp.to_rfc3339())]),
            )
            .await;
        body["lsn"].as_str().expect("lsn missing").to_string()
    }

    pub async fn restore(
        &self,
        organization_id: &str,
        project_id: &str,
        branch_id: &str,
        lsn: &str,
    ) -> (StatusCode, Value) {
        self.send(
            self.client
                .post(format!(
                    "{BASE_URL}/organizations/{organization_id}/projects/{project_id}/branches/{branch_id}/restore"
                ))
                .json(&json!({ "lsn": lsn })),
        )
        .await
    }

    pub async fn reset(&self, organization_id: &str, project_id: &str, branch_id: &str) -> (StatusCode, Value) {
        self.send(self.client.post(format!(
            "{BASE_URL}/organizations/{organization_id}/projects/{project_id}/branches/{branch_id}/reset"
        )))
        .await
    }
}

pub async fn connect(branch: &Value) -> tokio_postgres::Client {
    let port = branch["port"]
        .as_u64()
        .or_else(|| {
            branch["connection_string"]
                .as_str()
                .and_then(|value| value.rsplit(':').next())
                .and_then(|tail| tail.split('/').next())
                .and_then(|port| port.parse::<u64>().ok())
        })
        .expect("endpoint port missing");
    let password = branch["password"].as_str().expect("password missing");
    let config = format!("host=127.0.0.1 port={port} user=postgres password={password} dbname=postgres");

    let deadline = std::time::Instant::now() + Duration::from_secs(120);
    loop {
        match tokio_postgres::connect(&config, NoTls).await {
            Ok((client, connection)) => {
                tokio::spawn(async move {
                    let _ = connection.await;
                });
                return client;
            }
            Err(error) => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "could not connect to endpoint on port {port}: {error}"
                );
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

pub async fn wait_for_ingest(
    api: &Api,
    organization_id: &str,
    project_id: &str,
    branch_id: &str,
    client: &tokio_postgres::Client,
) {
    let flushed: String = client
        .query_one("SELECT pg_current_wal_flush_lsn()::text", &[])
        .await
        .expect("failed to read flush lsn")
        .get(0);
    let target = parse_lsn(&flushed);

    let deadline = std::time::Instant::now() + Duration::from_secs(120);
    loop {
        let branch = api.get_branch(organization_id, project_id, branch_id).await;
        let ingested = parse_lsn(branch["last_record_lsn"].as_str().expect("last_record_lsn missing"));
        if ingested >= target {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "pageserver did not ingest wal up to {flushed} for branch {branch_id}"
        );
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn parse_lsn(value: &str) -> u64 {
    let (high, low) = value.split_once('/').unwrap_or_else(|| panic!("malformed lsn {value}"));
    let high = u64::from_str_radix(high, 16).expect("malformed lsn high half");
    let low = u64::from_str_radix(low, 16).expect("malformed lsn low half");
    high << 32 | low
}

pub async fn seed(client: &tokio_postgres::Client, target_megabytes: i64) {
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS payload (id bigserial primary key, chunk uuid, body text)",
        )
        .await
        .expect("failed to create payload table");

    loop {
        let size: i64 = client
            .query_one("SELECT pg_total_relation_size('payload')::bigint", &[])
            .await
            .expect("failed to read relation size")
            .get(0);
        if size >= target_megabytes * 1024 * 1024 {
            println!("[seed] payload is {} MB", size / 1024 / 1024);
            break;
        }
        append(client, 200_000).await;
    }
}

pub async fn append(client: &tokio_postgres::Client, rows: i32) {
    client
        .execute(
            "INSERT INTO payload (chunk, body) \
             SELECT gen_random_uuid(), repeat(md5(random()::text), 32) FROM generate_series(1, $1)",
            &[&rows],
        )
        .await
        .expect("failed to insert payload rows");
}

pub async fn checksum(client: &tokio_postgres::Client) -> String {
    let row = client
        .query_one(
            "SELECT count(*)::bigint, coalesce(md5(string_agg(body, '' ORDER BY id)), '') FROM payload",
            &[],
        )
        .await
        .expect("failed to compute checksum");
    let count: i64 = row.get(0);
    let digest: String = row.get(1);
    format!("{count}:{digest}")
}
