use std::io::Write;
use std::process::{Command, Stdio};
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

/// Number of WAL segment files the safekeeper keeps on disk for a timeline.
pub fn safekeeper_wal_segments(project_id: &str, timeline_id: &str) -> usize {
    let directory = format!(
        "/neond/neon_daemon_data/safekeeper/{}/{}",
        project_id.replace('-', ""),
        timeline_id.replace('-', "")
    );
    let output = compose(&["exec", "-T", "neond", "ls", &directory]);
    assert!(
        output.status.success(),
        "failed to list {directory}:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|name| name.len() >= 24 && name[..24].chars().all(|c| c.is_ascii_hexdigit()))
        .count()
}

pub fn pg_tool_in_container(pg_version: &str, tool: &str, arguments: &[&str], input: &[u8]) -> std::process::Output {
    let install = format!("/usr/local/share/neon/pg_install/{pg_version}");
    let mut child = Command::new("docker")
        .arg("compose")
        .arg("-f")
        .arg(COMPOSE_FILE)
        .args(["exec", "-T", "-e", &format!("LD_LIBRARY_PATH={install}/lib"), "neond"])
        .arg(format!("{install}/bin/{tool}"))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to run docker compose exec");
    let mut stdin = child.stdin.take().expect("stdin unavailable");
    let input = input.to_vec();
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(&input);
    });
    let output = child.wait_with_output().expect("failed to wait for docker compose exec");
    writer.join().expect("stdin writer panicked");
    output
}

pub fn restore_in_container(dump: &[u8], format: &str, pg_version: &str, endpoint: &Value) {
    let port = endpoint_port(endpoint);
    let password = endpoint["password"].as_str().expect("password missing");
    let url = format!("postgresql://postgres:{password}@127.0.0.1:{port}/postgres");
    let output = match format {
        "custom" => pg_tool_in_container(
            pg_version,
            "pg_restore",
            &["--no-owner", "--no-privileges", "--exit-on-error", "-d", &url],
            dump,
        ),
        _ => pg_tool_in_container(pg_version, "psql", &["-X", "-q", "-v", "ON_ERROR_STOP=1", "-d", &url], dump),
    };
    assert!(
        output.status.success(),
        "restoring the {format} export with {pg_version} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
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

    pub fn token(&self) -> &str {
        &self.token
    }

    pub async fn other_user(&self, email: &str) -> Api {
        let (status, body) = self
            .send(self.client.post(format!("{BASE_URL}/auth/users")).json(&json!({
                "name": "E2E Other User",
                "email": email,
                "password": "e2e_password_123"
            })))
            .await;
        assert!(
            status.is_success() || status == StatusCode::CONFLICT,
            "creating user {email} failed with {status}: {body}"
        );

        let client = Client::new();
        let login = client
            .post(format!("{BASE_URL}/auth/login"))
            .json(&json!({ "email": email, "password": "e2e_password_123" }))
            .send()
            .await
            .expect("failed to send login request");
        assert!(login.status().is_success(), "login of {email} failed: {}", login.status());
        let token = login.json::<Value>().await.expect("bad login body")["token"]
            .as_str()
            .expect("token missing")
            .to_string();
        Api { client, token }
    }

    pub async fn create_project(&self, organization_id: &str, name: &str) -> String {
        self.create_project_with(organization_id, json!({ "name": name })).await
    }

    pub async fn create_project_with_version(&self, organization_id: &str, name: &str, pg_version: &str) -> String {
        self.create_project_with(organization_id, json!({ "name": name, "pg_version": pg_version }))
            .await
    }

    async fn create_project_with(&self, organization_id: &str, body: Value) -> String {
        let deadline = std::time::Instant::now() + Duration::from_secs(60);
        loop {
            let (status, body) = self
                .send(
                    self.client
                        .post(format!("{BASE_URL}/organizations/{organization_id}/projects"))
                        .json(&body),
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

    pub async fn update_project(&self, organization_id: &str, project_id: &str, body: Value) -> Value {
        self.expect_ok(
            self.client
                .put(format!("{BASE_URL}/organizations/{organization_id}/projects/{project_id}"))
                .json(&body),
        )
        .await
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

    pub async fn stop_running_endpoints(&self, organization_id: &str, project_id: &str) {
        let branches = self
            .expect_ok(self.client.get(format!(
                "{BASE_URL}/organizations/{organization_id}/projects/{project_id}/branches"
            )))
            .await;
        for branch in branches.as_array().expect("branch list is not an array") {
            if branch["endpoint_status"] == "running" {
                self.stop_endpoint(organization_id, project_id, branch["id"].as_str().unwrap())
                    .await;
            }
        }
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

    pub async fn compact(&self, organization_id: &str, project_id: &str, branch_id: &str) -> (StatusCode, Value) {
        self.send(self.client.post(format!(
            "{BASE_URL}/organizations/{organization_id}/projects/{project_id}/branches/{branch_id}/compact"
        )))
        .await
    }

    pub async fn compaction_status(&self, organization_id: &str, project_id: &str, branch_id: &str) -> Value {
        self.expect_ok(self.client.get(format!(
            "{BASE_URL}/organizations/{organization_id}/projects/{project_id}/branches/{branch_id}/compact"
        )))
        .await
    }

    pub async fn delete_branch(&self, organization_id: &str, project_id: &str, branch_id: &str) {
        self.expect_ok(self.client.delete(format!(
            "{BASE_URL}/organizations/{organization_id}/projects/{project_id}/branches/{branch_id}"
        )))
        .await;
    }

    pub async fn start_export(
        &self,
        organization_id: &str,
        project_id: &str,
        branch_id: &str,
        body: Value,
    ) -> (StatusCode, Value) {
        self.send(
            self.client
                .post(format!(
                    "{BASE_URL}/organizations/{organization_id}/projects/{project_id}/branches/{branch_id}/export"
                ))
                .json(&body),
        )
        .await
    }

    pub async fn export_status_raw(&self, organization_id: &str, project_id: &str, branch_id: &str) -> (StatusCode, Value) {
        self.send(self.client.get(format!(
            "{BASE_URL}/organizations/{organization_id}/projects/{project_id}/branches/{branch_id}/export"
        )))
        .await
    }

    pub async fn export_status(&self, organization_id: &str, project_id: &str, branch_id: &str) -> Value {
        let (status, body) = self.export_status_raw(organization_id, project_id, branch_id).await;
        assert!(status.is_success(), "export status failed with {status}: {body}");
        body
    }

    pub async fn cancel_export(&self, organization_id: &str, project_id: &str, branch_id: &str) -> (StatusCode, Value) {
        self.send(self.client.delete(format!(
            "{BASE_URL}/organizations/{organization_id}/projects/{project_id}/branches/{branch_id}/export"
        )))
        .await
    }

    pub async fn export_download_token(
        &self,
        organization_id: &str,
        project_id: &str,
        branch_id: &str,
    ) -> (StatusCode, Value) {
        self.send(self.client.post(format!(
            "{BASE_URL}/organizations/{organization_id}/projects/{project_id}/branches/{branch_id}/export/download-token"
        )))
        .await
    }

    pub async fn new_download_token(&self, organization_id: &str, project_id: &str, branch_id: &str) -> String {
        let (status, body) = self.export_download_token(organization_id, project_id, branch_id).await;
        assert!(status.is_success(), "download token request failed with {status}: {body}");
        body["token"].as_str().expect("download token missing").to_string()
    }

    pub async fn download_export(
        &self,
        organization_id: &str,
        project_id: &str,
        branch_id: &str,
        token: Option<&str>,
    ) -> reqwest::Response {
        let mut request = self.client.get(format!(
            "{BASE_URL}/organizations/{organization_id}/projects/{project_id}/branches/{branch_id}/export/download"
        ));
        if let Some(token) = token {
            request = request.query(&[("token", token)]);
        }
        request.send().await.expect("download request failed")
    }

    pub async fn me_with_bearer(&self, token: &str) -> StatusCode {
        self.client
            .get(format!("{BASE_URL}/auth/me"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .expect("request failed")
            .status()
    }
}

fn is_export_active(state: &str) -> bool {
    matches!(state, "preparing" | "dumping" | "finalizing")
}

pub async fn wait_for_export(
    api: &Api,
    organization_id: &str,
    project_id: &str,
    branch_id: &str,
) -> (Value, Vec<Value>) {
    let deadline = std::time::Instant::now() + Duration::from_secs(1200);
    let mut history = Vec::new();
    loop {
        let status = api.export_status(organization_id, project_id, branch_id).await;
        let state = status["state"].as_str().unwrap_or_default().to_string();
        if !is_export_active(&state) {
            return (status, history);
        }
        history.push(status.clone());
        assert!(
            std::time::Instant::now() < deadline,
            "export did not finish in time: {status}"
        );
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

pub async fn wait_for_export_state(
    api: &Api,
    organization_id: &str,
    project_id: &str,
    branch_id: &str,
    state: &str,
) -> Value {
    let deadline = std::time::Instant::now() + Duration::from_secs(300);
    loop {
        let status = api.export_status(organization_id, project_id, branch_id).await;
        if status["state"] == state {
            return status;
        }
        assert!(
            std::time::Instant::now() < deadline && is_export_active(status["state"].as_str().unwrap_or_default()),
            "export did not reach state {state}: {status}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub fn endpoint_port(endpoint: &Value) -> u64 {
    endpoint["port"]
        .as_u64()
        .or_else(|| {
            endpoint["connection_string"]
                .as_str()
                .and_then(|value| value.rsplit(':').next())
                .and_then(|tail| tail.split('/').next())
                .and_then(|port| port.parse::<u64>().ok())
        })
        .expect("endpoint port missing")
}

/// Waits for the storage compaction of a branch to finish and returns its final status.
pub async fn wait_for_compaction(api: &Api, organization_id: &str, project_id: &str, branch_id: &str) -> Value {
    let deadline = std::time::Instant::now() + Duration::from_secs(1200);
    loop {
        let status = api.compaction_status(organization_id, project_id, branch_id).await;
        if status["running"] == false {
            return status;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "compaction did not finish in time: {status}"
        );
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

pub async fn connect(branch: &Value) -> tokio_postgres::Client {
    let port = endpoint_port(branch);
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
