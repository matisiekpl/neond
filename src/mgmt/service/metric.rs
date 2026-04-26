use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use chrono::{NaiveDateTime, Utc};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::dto::metric_snapshot::{BranchLabelMap, BranchLabels, LatestMetricKey, LatestMetrics, MetricSnapshot};
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

pub const PAGESERVER_TIMELINE_LAYER_BYTES: &str = "pageserver.timeline.layer_bytes";
pub const PAGESERVER_TIMELINE_LAYER_COUNT: &str = "pageserver.timeline.layer_count";
pub const PAGESERVER_TIMELINE_EPHEMERAL_BYTES: &str = "pageserver.timeline.ephemeral_bytes";
pub const PAGESERVER_TIMELINE_DIRECTORY_ENTRIES: &str = "pageserver.timeline.directory_entries";
pub const PAGESERVER_TIMELINE_WAIT_LSN_SECONDS_SUM: &str = "pageserver.timeline.wait_lsn_seconds_sum";
pub const PAGESERVER_TIMELINE_ONDEMAND_DOWNLOAD_BYTES: &str = "pageserver.timeline.ondemand_download_bytes_total";
pub const PAGESERVER_TIMELINE_SMGR_QUERY_STARTED: &str = "pageserver.timeline.smgr_query_started_count";

pub const PAGESERVER_PAGE_CACHE_HITS: &str = "pageserver.page_cache.hits_total";
pub const PAGESERVER_PAGE_CACHE_ACCESSES: &str = "pageserver.page_cache.accesses_total";
pub const PAGESERVER_PAGE_CACHE_SIZE_BYTES: &str = "pageserver.page_cache.size_bytes";
pub const PAGESERVER_TENANT_STATES_COUNT: &str = "pageserver.tenant_states_count";
pub const PAGESERVER_BROKEN_TENANTS_COUNT: &str = "pageserver.broken_tenants_count";
pub const PAGESERVER_IO_OPERATIONS_BYTES: &str = "pageserver.io_operations_bytes_total";

pub const SAFEKEEPER_WRITE_WAL_BYTES: &str = "safekeeper.write_wal_bytes";
pub const SAFEKEEPER_FLUSH_WAL_SECONDS_SUM: &str = "safekeeper.flush_wal_seconds_sum";
pub const SAFEKEEPER_WAL_RECEIVERS: &str = "safekeeper.wal_receivers";
pub const SAFEKEEPER_WAL_READERS: &str = "safekeeper.wal_readers";
pub const SAFEKEEPER_REMOVED_WAL_SEGMENTS: &str = "safekeeper.removed_wal_segments_total";
pub const SAFEKEEPER_BACKED_UP_SEGMENTS: &str = "safekeeper.backed_up_segments_total";
pub const SAFEKEEPER_BACKUP_ERRORS: &str = "safekeeper.backup_errors_total";
pub const SAFEKEEPER_EVICTED_TIMELINES: &str = "safekeeper.evicted_timelines";
pub const SAFEKEEPER_PROPOSER_ACCEPTOR_MESSAGES: &str = "safekeeper.proposer_acceptor_messages_total";
pub const SAFEKEEPER_PARTIAL_BACKUP_UPLOADED_BYTES: &str = "safekeeper.partial_backup_uploaded_bytes_total";

const COLLECTION_INTERVAL: Duration = Duration::from_secs(10);
const CLEANUP_INTERVAL: Duration = Duration::from_secs(300);
const RETENTION_HOURS: i64 = 24;
const SCRAPE_TIMEOUT: Duration = Duration::from_secs(2);
const SQL_TIMEOUT: Duration = Duration::from_secs(2);

const PAGESERVER_METRICS_URL: &str = "http://127.0.0.1:9898/metrics";
const SAFEKEEPER_METRICS_URL: &str = "http://127.0.0.1:7676/metrics";

pub struct MetricService {
    metric_repo: Arc<MetricRepository>,
    endpoint_service: Arc<EndpointService>,
    branch_repo: Arc<BranchRepository>,
    project_repo: Arc<ProjectRepository>,
    membership_service: Arc<MembershipService>,
    http_client: reqwest::Client,
    system: Arc<Mutex<System>>,
    snapshot: Arc<RwLock<MetricSnapshot>>,
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
            snapshot: Arc::new(RwLock::new(MetricSnapshot::default())),
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

    pub async fn list_daemon(
        &self,
        from: NaiveDateTime,
        to: NaiveDateTime,
    ) -> Result<Vec<ComputeMetricSample>> {
        self.metric_repo.list_daemon(from, to).await
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

        self.refresh_processes().await;

        let recorded_at = Utc::now().naive_utc();
        let mut batch: Vec<NewComputeMetricSample> = Vec::new();
        let mut latest_samples: LatestMetrics = HashMap::new();
        let mut seen_branch_ids: HashSet<Uuid> = HashSet::new();

        for target in &targets {
            let process_samples = self.sample_process(target).await;
            let http_samples = self.sample_compute_ctl(target).await;
            let sql_samples = self.sample_pg_connections(target).await;

            seen_branch_ids.insert(target.branch_id);

            for (slug, value) in process_samples
                .into_iter()
                .chain(http_samples.into_iter())
                .chain(sql_samples.into_iter())
            {
                latest_samples.insert(
                    LatestMetricKey { slug: slug.to_string(), branch_id: Some(target.branch_id) },
                    value,
                );
                batch.push(NewComputeMetricSample {
                    id: Uuid::new_v4(),
                    branch_id: Some(target.branch_id),
                    recorded_at,
                    slug: slug.to_string(),
                    value,
                });
            }
        }

        let pageserver_timeline_samples = self.sample_pageserver_timeline_metrics().await;
        for (branch_id, slug, value) in pageserver_timeline_samples {
            seen_branch_ids.insert(branch_id);
            latest_samples.insert(
                LatestMetricKey { slug: slug.clone(), branch_id: Some(branch_id) },
                value,
            );
            batch.push(NewComputeMetricSample {
                id: Uuid::new_v4(),
                branch_id: Some(branch_id),
                recorded_at,
                slug,
                value,
            });
        }

        let pageserver_global_samples = self.sample_pageserver_global_metrics().await;
        for (slug, value) in pageserver_global_samples {
            latest_samples.insert(
                LatestMetricKey { slug: slug.to_string(), branch_id: None },
                value,
            );
            batch.push(NewComputeMetricSample {
                id: Uuid::new_v4(),
                branch_id: None,
                recorded_at,
                slug: slug.to_string(),
                value,
            });
        }

        let safekeeper_samples = self.sample_safekeeper_metrics().await;
        for (slug, value) in safekeeper_samples {
            latest_samples.insert(
                LatestMetricKey { slug: slug.to_string(), branch_id: None },
                value,
            );
            batch.push(NewComputeMetricSample {
                id: Uuid::new_v4(),
                branch_id: None,
                recorded_at,
                slug: slug.to_string(),
                value,
            });
        }

        let branch_labels = self.build_branch_labels(&seen_branch_ids).await;

        *self.snapshot.write().await = MetricSnapshot { samples: latest_samples, branch_labels };

        if let Err(error) = self.metric_repo.insert_batch(batch).await {
            tracing::warn!("Metric batch insert failed: {}", error);
        }
    }

    async fn build_branch_labels(&self, branch_ids: &HashSet<Uuid>) -> BranchLabelMap {
        let mut labels: BranchLabelMap = HashMap::new();
        for &branch_id in branch_ids {
            let branch = match self.branch_repo.find_by_id(branch_id).await {
                Ok(Some(branch)) => branch,
                _ => continue,
            };
            let project = match self.project_repo.find_by_id(branch.project_id).await {
                Ok(Some(project)) => project,
                _ => continue,
            };
            labels.insert(branch_id, BranchLabels {
                branch_name: branch.name,
                project_name: project.name,
            });
        }
        labels
    }

    pub async fn snapshot_latest(&self) -> MetricSnapshot {
        self.snapshot.read().await.clone()
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
        let compute_ctl_pid = Pid::from_u32(target.pid);

        let postgres_root = system
            .processes()
            .iter()
            .find(|(_, process)| {
                process.parent() == Some(compute_ctl_pid)
                    && process
                        .name()
                        .to_string_lossy()
                        .to_lowercase()
                        .contains("postgres")
            })
            .map(|(pid, _)| *pid);

        let Some(postgres_root) = postgres_root else {
            return Vec::new();
        };

        let mut pids_to_include: HashSet<Pid> = HashSet::new();
        pids_to_include.insert(postgres_root);

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

    async fn fetch_metrics_body(&self, url: &str) -> Option<String> {
        match self.http_client.get(url).send().await {
            Ok(response) => response.text().await.ok(),
            Err(error) => {
                tracing::debug!("Failed to fetch {}: {}", url, error);
                None
            }
        }
    }

    fn parse_prometheus_labels(labels_str: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for pair in labels_str.split(',') {
            if let Some((key, rest)) = pair.split_once('=') {
                let value = rest.trim_matches('"');
                map.insert(key.trim().to_string(), value.to_string());
            }
        }
        map
    }

    async fn sample_pageserver_timeline_metrics(&self) -> Vec<(Uuid, String, f64)> {
        let body = match self.fetch_metrics_body(PAGESERVER_METRICS_URL).await {
            Some(body) => body,
            None => return Vec::new(),
        };

        let mut aggregated: HashMap<(Uuid, &'static str), f64> = HashMap::new();

        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let mut tokens = trimmed.splitn(2, ' ');
            let name_with_labels = match tokens.next() {
                Some(value) => value,
                None => continue,
            };
            let value_str = match tokens.next() {
                Some(value) => value.split_whitespace().next().unwrap_or(""),
                None => continue,
            };
            let value: f64 = match value_str.parse() {
                Ok(v) => v,
                Err(_) => continue,
            };

            let (name, labels_str) = match name_with_labels.split_once('{') {
                Some((name, rest)) => (name, rest.trim_end_matches('}')),
                None => continue,
            };

            let labels = Self::parse_prometheus_labels(labels_str);
            let timeline_id_str = match labels.get("timeline_id") {
                Some(value) => value,
                None => continue,
            };
            let timeline_id = match Uuid::parse_str(timeline_id_str) {
                Ok(uuid) => uuid,
                Err(_) => continue,
            };

            let slug: Option<&'static str> = match name {
                "pageserver_layer_bytes" => Some(PAGESERVER_TIMELINE_LAYER_BYTES),
                "pageserver_layer_count" => Some(PAGESERVER_TIMELINE_LAYER_COUNT),
                "pageserver_timeline_ephemeral_bytes" => Some(PAGESERVER_TIMELINE_EPHEMERAL_BYTES),
                "pageserver_directory_entries_count" => Some(PAGESERVER_TIMELINE_DIRECTORY_ENTRIES),
                "pageserver_wait_lsn_seconds_sum" => Some(PAGESERVER_TIMELINE_WAIT_LSN_SECONDS_SUM),
                "pageserver_ondemand_download_bytes_total" => Some(PAGESERVER_TIMELINE_ONDEMAND_DOWNLOAD_BYTES),
                "pageserver_smgr_query_started_count" => Some(PAGESERVER_TIMELINE_SMGR_QUERY_STARTED),
                _ => None,
            };

            if let Some(slug) = slug {
                let entry = aggregated.entry((timeline_id, slug)).or_insert(0.0);
                *entry += value;
            }
        }

        let mut result = Vec::new();
        for ((timeline_id, slug), value) in aggregated {
            match self.branch_repo.find_by_timeline_id(timeline_id).await {
                Ok(Some(branch)) => {
                    result.push((branch.id, slug.to_string(), value));
                }
                Ok(None) => {}
                Err(error) => {
                    tracing::debug!("Failed to resolve timeline {} to branch: {}", timeline_id, error);
                }
            }
        }
        result
    }

    async fn sample_pageserver_global_metrics(&self) -> Vec<(&'static str, f64)> {
        let body = match self.fetch_metrics_body(PAGESERVER_METRICS_URL).await {
            Some(body) => body,
            None => return Vec::new(),
        };

        let mut aggregated: HashMap<&'static str, f64> = HashMap::new();

        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let mut tokens = trimmed.splitn(2, ' ');
            let name_with_labels = match tokens.next() {
                Some(value) => value,
                None => continue,
            };
            let value_str = match tokens.next() {
                Some(value) => value.split_whitespace().next().unwrap_or(""),
                None => continue,
            };
            let value: f64 = match value_str.parse() {
                Ok(v) => v,
                Err(_) => continue,
            };

            let name = name_with_labels
                .split_once('{')
                .map(|(base, _)| base)
                .unwrap_or(name_with_labels);

            let has_timeline_label = name_with_labels.contains("timeline_id");
            if has_timeline_label {
                continue;
            }

            let slug: Option<&'static str> = match name {
                "pageserver_page_cache_read_hits_total" => Some(PAGESERVER_PAGE_CACHE_HITS),
                "pageserver_page_cache_read_accesses_total" => Some(PAGESERVER_PAGE_CACHE_ACCESSES),
                "pageserver_page_cache_size_current_bytes" => Some(PAGESERVER_PAGE_CACHE_SIZE_BYTES),
                "pageserver_tenant_states_count" => Some(PAGESERVER_TENANT_STATES_COUNT),
                "pageserver_broken_tenants_count" => Some(PAGESERVER_BROKEN_TENANTS_COUNT),
                "pageserver_io_operations_bytes_total" => Some(PAGESERVER_IO_OPERATIONS_BYTES),
                _ => None,
            };

            if let Some(slug) = slug {
                let entry = aggregated.entry(slug).or_insert(0.0);
                *entry += value;
            }
        }

        aggregated.into_iter().collect()
    }

    async fn sample_safekeeper_metrics(&self) -> Vec<(&'static str, f64)> {
        let body = match self.fetch_metrics_body(SAFEKEEPER_METRICS_URL).await {
            Some(body) => body,
            None => return Vec::new(),
        };

        let mut aggregated: HashMap<&'static str, f64> = HashMap::new();

        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let mut tokens = trimmed.splitn(2, ' ');
            let name_with_labels = match tokens.next() {
                Some(value) => value,
                None => continue,
            };
            let value_str = match tokens.next() {
                Some(value) => value.split_whitespace().next().unwrap_or(""),
                None => continue,
            };
            let value: f64 = match value_str.parse() {
                Ok(v) => v,
                Err(_) => continue,
            };

            let name = name_with_labels
                .split_once('{')
                .map(|(base, _)| base)
                .unwrap_or(name_with_labels);

            let slug: Option<&'static str> = match name {
                "safekeeper_write_wal_bytes" => Some(SAFEKEEPER_WRITE_WAL_BYTES),
                "safekeeper_flush_wal_seconds_sum" => Some(SAFEKEEPER_FLUSH_WAL_SECONDS_SUM),
                "safekeeper_wal_receivers" => Some(SAFEKEEPER_WAL_RECEIVERS),
                "safekeeper_wal_readers" => Some(SAFEKEEPER_WAL_READERS),
                "safekeeper_removed_wal_segments_total" => Some(SAFEKEEPER_REMOVED_WAL_SEGMENTS),
                "safekeeper_backed_up_segments_total" => Some(SAFEKEEPER_BACKED_UP_SEGMENTS),
                "safekeeper_backup_errors_total" => Some(SAFEKEEPER_BACKUP_ERRORS),
                "safekeeper_evicted_timelines" => Some(SAFEKEEPER_EVICTED_TIMELINES),
                "safekeeper_proposer_acceptor_messages_total" => Some(SAFEKEEPER_PROPOSER_ACCEPTOR_MESSAGES),
                "safekeeper_partial_backup_uploaded_bytes_total" => Some(SAFEKEEPER_PARTIAL_BACKUP_UPLOADED_BYTES),
                _ => None,
            };

            if let Some(slug) = slug {
                let entry = aggregated.entry(slug).or_insert(0.0);
                *entry += value;
            }
        }

        aggregated.into_iter().collect()
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