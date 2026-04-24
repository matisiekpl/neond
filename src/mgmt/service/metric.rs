use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use chrono::{NaiveDateTime, Utc};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::dto::metric_target::MetricTarget;
use crate::mgmt::model::compute_metric_sample::{ComputeMetricSample, NewComputeMetricSample};
use crate::mgmt::repository::branch::BranchRepository;
use crate::mgmt::repository::metric::MetricRepository;
use crate::mgmt::repository::project::ProjectRepository;
use crate::mgmt::service::endpoint::EndpointService;
use crate::mgmt::service::membership::MembershipService;

pub const CPU_PERCENT: &str = "cpu.percent";
pub const MEM_RSS: &str = "mem.rss";
pub const MEM_VSZ: &str = "mem.vsz";

pub const PG_CONNECTIONS_TOTAL: &str = "pg.connections.total";
pub const PG_CONNECTIONS_ACTIVE: &str = "pg.connections.active";
pub const PG_CONNECTIONS_IDLE: &str = "pg.connections.idle";

pub const COMPUTE_CTL_UP: &str = "compute_ctl.up";
pub const COMPUTE_CTL_PG_DOWNTIME_MS: &str = "compute_ctl.pg_downtime_ms";
pub const COMPUTE_CTL_PAGESTREAM_ERRORS_TOTAL: &str = "compute_ctl.pagestream_errors_total";

const COLLECTION_INTERVAL: Duration = Duration::from_secs(10);
const CLEANUP_INTERVAL: Duration = Duration::from_secs(300);
const RETENTION_HOURS: i64 = 24;
const SCRAPE_TIMEOUT: Duration = Duration::from_secs(2);
const SQL_TIMEOUT: Duration = Duration::from_secs(2);

pub struct MetricService {
    metric_repo: Arc<MetricRepository>,
    endpoint_service: Arc<EndpointService>,
    branch_repo: Arc<BranchRepository>,
    project_repo: Arc<ProjectRepository>,
    membership_service: Arc<MembershipService>,
    http_client: reqwest::Client,
    system: Arc<Mutex<System>>,
}

impl MetricService {
    pub fn new(
        metric_repo: Arc<MetricRepository>,
        endpoint_service: Arc<EndpointService>,
        branch_repo: Arc<BranchRepository>,
        project_repo: Arc<ProjectRepository>,
        membership_service: Arc<MembershipService>,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(SCRAPE_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            metric_repo,
            endpoint_service,
            branch_repo,
            project_repo,
            membership_service,
            http_client,
            system: Arc::new(Mutex::new(System::new())),
        }
    }

    pub async fn list(
        &self,
        user_id: Uuid,
        organization_id: Uuid,
        project_id: Uuid,
        branch_id: Uuid,
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> Result<Vec<ComputeMetricSample>> {
        self.membership_service
            .verify_membership(user_id, organization_id)
            .await?;

        let project = self
            .project_repo
            .find_by_id(project_id)
            .await?
            .ok_or(AppError::NotFound)?;
        if project.organization_id != organization_id {
            return Err(AppError::NotFound);
        }

        let branch = self
            .branch_repo
            .find_by_id(branch_id)
            .await?
            .ok_or(AppError::NotFound)?;
        if branch.project_id != project_id {
            return Err(AppError::NotFound);
        }

        self.metric_repo
            .list_for_branch(branch_id, from, to)
            .await
    }

    pub async fn cleanup(&self) {
        let cutoff = Utc::now().naive_utc() - chrono::Duration::hours(RETENTION_HOURS);
        match self.metric_repo.delete_older_than(cutoff).await {
            Ok(deleted) if deleted > 0 => {
                tracing::info!("Pruned {} expired metric samples", deleted);
            }
            Ok(_) => {}
            Err(error) => {
                tracing::warn!("Metric cleanup failed: {}", error);
            }
        }
    }

    pub async fn collect_all(&self) {
        let targets = self.endpoint_service.get_running_targets().await;
        if targets.is_empty() {
            return;
        }

        self.refresh_processes().await;

        let recorded_at = Utc::now().naive_utc();
        let mut batch: Vec<NewComputeMetricSample> = Vec::new();

        for target in &targets {
            let process_samples = self.sample_process(target).await;
            let http_samples = self.sample_compute_ctl(target).await;
            let sql_samples = self.sample_pg_connections(target).await;

            for (slug, value) in process_samples
                .into_iter()
                .chain(http_samples.into_iter())
                .chain(sql_samples.into_iter())
            {
                batch.push(NewComputeMetricSample {
                    id: Uuid::new_v4(),
                    branch_id: target.branch_id,
                    recorded_at,
                    slug: slug.to_string(),
                    value,
                });
            }
        }

        if let Err(error) = self.metric_repo.insert_batch(batch).await {
            tracing::warn!("Metric batch insert failed: {}", error);
        }
    }

    async fn refresh_processes(&self) {
        let mut system = self.system.lock().await;
        system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::new().with_cpu().with_memory(),
        );
    }

    async fn sample_process(&self, target: &MetricTarget) -> Vec<(&'static str, f64)> {
        let system = self.system.lock().await;
        let root_pid = Pid::from_u32(target.pid);

        let mut pids_to_include: HashSet<Pid> = HashSet::new();
        pids_to_include.insert(root_pid);

        let mut changed = true;
        while changed {
            changed = false;
            for (child_pid, child_process) in system.processes() {
                if pids_to_include.contains(child_pid) {
                    continue;
                }
                if let Some(parent_pid) = child_process.parent() {
                    if pids_to_include.contains(&parent_pid) {
                        pids_to_include.insert(*child_pid);
                        changed = true;
                    }
                }
            }
        }

        let mut total_cpu: f64 = 0.0;
        let mut total_rss: u64 = 0;
        let mut total_vsz: u64 = 0;

        for pid in &pids_to_include {
            if let Some(process) = system.process(*pid) {
                total_cpu += process.cpu_usage() as f64;
                total_rss += process.memory();
                total_vsz += process.virtual_memory();
            }
        }

        if total_rss == 0 && total_cpu == 0.0 {
            return Vec::new();
        }

        vec![
            (CPU_PERCENT, total_cpu),
            (MEM_RSS, total_rss as f64),
            (MEM_VSZ, total_vsz as f64),
        ]
    }

    async fn sample_compute_ctl(&self, target: &MetricTarget) -> Vec<(&'static str, f64)> {
        let url = format!("http://127.0.0.1:{}/metrics", target.metrics_port);
        let body = match self.http_client.get(&url).send().await {
            Ok(response) => match response.text().await {
                Ok(text) => text,
                Err(error) => {
                    tracing::debug!(
                        "Failed to read compute_ctl /metrics body for branch {}: {}",
                        target.branch_id,
                        error
                    );
                    return Vec::new();
                }
            },
            Err(error) => {
                tracing::debug!(
                    "Failed to fetch compute_ctl /metrics for branch {}: {}",
                    target.branch_id,
                    error
                );
                return Vec::new();
            }
        };

        let mut parsed: HashMap<&'static str, f64> = HashMap::new();
        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let mut tokens = trimmed.split_whitespace();
            let name_with_labels = match tokens.next() {
                Some(value) => value,
                None => continue,
            };
            let value_token = match tokens.next() {
                Some(value) => value,
                None => continue,
            };
            let value: f64 = match value_token.parse() {
                Ok(parsed_value) => parsed_value,
                Err(_) => continue,
            };
            let name = name_with_labels
                .split_once('{')
                .map(|(base, _)| base)
                .unwrap_or(name_with_labels);

            match name {
                "compute_ctl_up" => {
                    parsed.insert(COMPUTE_CTL_UP, value);
                }
                "compute_pg_current_downtime_ms" => {
                    parsed.insert(COMPUTE_CTL_PG_DOWNTIME_MS, value);
                }
                "pg_cctl_pagestream_request_errors_total" => {
                    let entry = parsed
                        .entry(COMPUTE_CTL_PAGESTREAM_ERRORS_TOTAL)
                        .or_insert(0.0);
                    *entry += value;
                }
                _ => {}
            }
        }

        parsed.into_iter().collect()
    }

    async fn sample_pg_connections(&self, target: &MetricTarget) -> Vec<(&'static str, f64)> {
        let connection_string = format!(
            "host=127.0.0.1 port={} user=cloud_admin dbname=postgres",
            target.pg_port
        );

        let connect_future =
            tokio_postgres::connect(&connection_string, tokio_postgres::NoTls);
        let connect_result = match tokio::time::timeout(SQL_TIMEOUT, connect_future).await {
            Ok(Ok(pair)) => pair,
            Ok(Err(error)) => {
                tracing::debug!(
                    "Failed to connect for pg_stat_activity on branch {}: {}",
                    target.branch_id,
                    error
                );
                return Vec::new();
            }
            Err(_) => {
                tracing::debug!(
                    "Timed out connecting for pg_stat_activity on branch {}",
                    target.branch_id
                );
                return Vec::new();
            }
        };
        let (client, connection) = connect_result;
        let connection_task = tokio::spawn(async move {
            if let Err(error) = connection.await {
                tracing::debug!("pg_stat_activity connection error: {}", error);
            }
        });

        let query_future = client.simple_query(
            "SELECT state, count(*) FROM pg_stat_activity WHERE state IS NOT NULL GROUP BY state",
        );
        let messages = match tokio::time::timeout(SQL_TIMEOUT, query_future).await {
            Ok(Ok(messages)) => messages,
            Ok(Err(error)) => {
                tracing::debug!(
                    "pg_stat_activity query failed on branch {}: {}",
                    target.branch_id,
                    error
                );
                drop(client);
                connection_task.abort();
                return Vec::new();
            }
            Err(_) => {
                tracing::debug!(
                    "pg_stat_activity query timed out on branch {}",
                    target.branch_id
                );
                drop(client);
                connection_task.abort();
                return Vec::new();
            }
        };
        drop(client);
        connection_task.abort();

        let mut active: f64 = 0.0;
        let mut idle: f64 = 0.0;
        let mut total: f64 = 0.0;

        for message in messages {
            if let tokio_postgres::SimpleQueryMessage::Row(row) = message {
                let state = row.get(0).unwrap_or("");
                let count: f64 = row.get(1).and_then(|value| value.parse().ok()).unwrap_or(0.0);
                total += count;
                match state {
                    "active" => active += count,
                    state if state.starts_with("idle") => idle += count,
                    _ => {}
                }
            }
        }

        vec![
            (PG_CONNECTIONS_TOTAL, total),
            (PG_CONNECTIONS_ACTIVE, active),
            (PG_CONNECTIONS_IDLE, idle),
        ]
    }

    pub async fn run_collection_loop(self: Arc<Self>, shutdown: CancellationToken) {
        let mut ticker = tokio::time::interval(COLLECTION_INTERVAL);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    tracing::info!("Metric collection loop stopping");
                    return;
                }
                _ = ticker.tick() => {
                    self.collect_all().await;
                }
            }
        }
    }

    pub async fn run_cleanup_loop(self: Arc<Self>, shutdown: CancellationToken) {
        let mut ticker = tokio::time::interval(CLEANUP_INTERVAL);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    tracing::info!("Metric cleanup loop stopping");
                    return;
                }
                _ = ticker.tick() => {
                    self.cleanup().await;
                }
            }
        }
    }
}
