use serde_json::{json, Value};

use crate::common::{
    api, checksum, connect, exclusive, pg_tool_in_container, restore_in_container, seed,
    wait_for_export, wait_for_export_state, Api,
};

const SEED_MEGABYTES: i64 = 300;

async fn create_extra_tables(client: &tokio_postgres::Client) {
    client
        .batch_execute(
            "CREATE TABLE customers (id serial PRIMARY KEY, name text NOT NULL);
             INSERT INTO customers (name) SELECT 'customer ' || n FROM generate_series(1, 5000) n;
             CREATE TABLE orders (id bigserial PRIMARY KEY, customer_id int REFERENCES customers(id), total numeric(12, 2));
             INSERT INTO orders (customer_id, total) SELECT 1 + n % 5000, n * 1.5 FROM generate_series(1, 200000) n;
             CREATE SCHEMA reporting;
             CREATE TABLE reporting.daily (day date PRIMARY KEY, revenue numeric);
             INSERT INTO reporting.daily SELECT current_date - n, n FROM generate_series(1, 365) n;",
        )
        .await
        .expect("failed to create extra tables");
}

async fn extra_tables_checksum(client: &tokio_postgres::Client) -> String {
    let row = client
        .query_one(
            "SELECT (SELECT count(*) FROM customers)::text || ':' ||
                    (SELECT sum(total) FROM orders)::text || ':' ||
                    (SELECT count(*) FROM reporting.daily)::text",
            &[],
        )
        .await
        .expect("failed to compute checksum of extra tables");
    row.get(0)
}

async fn download(api: &Api, organization_id: &str, project_id: &str, branch_id: &str) -> (Vec<u8>, reqwest::header::HeaderMap) {
    let token = api.new_download_token(organization_id, project_id, branch_id).await;
    let response = api
        .download_export(organization_id, project_id, branch_id, Some(&token))
        .await;
    assert_eq!(response.status().as_u16(), 200, "download should succeed");
    let headers = response.headers().clone();
    let bytes = response.bytes().await.expect("failed to read the export").to_vec();
    (bytes, headers)
}

fn assert_completed(status: &Value) {
    assert_eq!(status["state"], "completed", "export should complete: {status}");
    assert!(status["error"].is_null(), "export reported an error: {status}");
    assert_eq!(status["progress"].as_f64(), Some(1.0), "finished export must report full progress: {status}");
    assert_eq!(
        status["tables_done"], status["tables_total"],
        "every table should be counted as dumped: {status}"
    );
}

#[tokio::test]
async fn export_custom_format_roundtrips() {
    let _guard = exclusive().await;
    let api = api().await;

    let organization_id = api.create_organization("Export Org").await;
    let project_id = api.create_project(&organization_id, "Export Project").await;

    let source = api.create_branch(&organization_id, &project_id, "production", None).await;
    let source_id = source["id"].as_str().unwrap().to_string();
    let source_endpoint = api.start_endpoint(&organization_id, &project_id, &source_id).await;
    let source_client = connect(&source_endpoint).await;
    seed(&source_client, SEED_MEGABYTES).await;
    create_extra_tables(&source_client).await;
    let expected_payload = checksum(&source_client).await;
    let expected_extra = extra_tables_checksum(&source_client).await;

    let idle = api.export_status(&organization_id, &project_id, &source_id).await;
    assert_eq!(idle["state"], "idle", "a branch without exports should be idle: {idle}");
    let (status, body) = api.export_download_token(&organization_id, &project_id, &source_id).await;
    assert_eq!(status.as_u16(), 409, "no download without a finished export: {body}");

    let (status, started) = api
        .start_export(&organization_id, &project_id, &source_id, json!({ "format": "custom" }))
        .await;
    assert_eq!(status.as_u16(), 202, "export should be accepted: {started}");
    assert_eq!(started["state"], "preparing");
    assert_eq!(started["database"], "postgres");
    assert_eq!(started["pg_version"], "V17");

    let (status, body) = api
        .start_export(&organization_id, &project_id, &source_id, json!({ "format": "plain" }))
        .await;
    assert_eq!(status.as_u16(), 409, "a second export must be rejected while one runs: {body}");

    let (status, history) = wait_for_export(&api, &organization_id, &project_id, &source_id).await;
    assert_completed(&status);
    println!(
        "[export] {} bytes, {} tables, {} progress samples",
        status["bytes_written"], status["tables_total"], history.len()
    );
    assert!(
        status["tables_total"].as_u64().unwrap() >= 4,
        "payload, customers, orders and reporting.daily should be dumped: {status}"
    );
    assert!(
        status["pg_dump_version"].as_str().is_some_and(|version| version.starts_with("17.")),
        "the export should use pg_dump 17: {status}"
    );
    assert!(
        status["file_name"].as_str().is_some_and(|name| name.ends_with(".dump")),
        "custom exports are .dump files: {status}"
    );
    assert!(status["expires_at"].is_string(), "a finished export expires: {status}");

    let progress: Vec<f64> = history.iter().filter_map(|sample| sample["progress"].as_f64()).collect();
    assert!(
        progress.windows(2).all(|pair| pair[0] <= pair[1]),
        "progress must never go back: {progress:?}"
    );
    assert!(
        progress.iter().all(|value| (0.0..1.0).contains(value)),
        "progress stays below 100% until pg_dump has exited: {progress:?}"
    );
    assert!(
        history.iter().any(|sample| sample["state"] == "dumping" && sample["current_table"].is_string()),
        "the current table should be reported while dumping: {history:?}"
    );
    let written: Vec<u64> = history.iter().filter_map(|sample| sample["bytes_written"].as_u64()).collect();
    assert!(
        written.windows(2).all(|pair| pair[0] <= pair[1]),
        "written bytes must never go back: {written:?}"
    );

    let (dump, headers) = download(&api, &organization_id, &project_id, &source_id).await;
    assert_eq!(dump.len() as u64, status["bytes_written"].as_u64().unwrap());
    assert_eq!(
        headers["content-length"].to_str().unwrap(),
        dump.len().to_string(),
        "Content-Length should be the file size"
    );
    let disposition = headers["content-disposition"].to_str().unwrap();
    assert!(
        disposition.starts_with("attachment; filename=\"export-project-production-") && disposition.ends_with(".dump\""),
        "unexpected Content-Disposition: {disposition}"
    );
    assert_eq!(headers["x-content-type-options"], "nosniff");
    assert_eq!(headers["cache-control"], "no-store");
    assert!(dump.starts_with(b"PGDMP"), "a custom export is a pg_dump archive");

    let target = api.create_branch(&organization_id, &project_id, "restored", None).await;
    let target_id = target["id"].as_str().unwrap().to_string();
    let target_endpoint = api.start_endpoint(&organization_id, &project_id, &target_id).await;
    restore_in_container(&dump, "custom", "v17", &target_endpoint);

    let target_client = connect(&target_endpoint).await;
    assert_eq!(checksum(&target_client).await, expected_payload, "restored payload must match");
    assert_eq!(
        extra_tables_checksum(&target_client).await,
        expected_extra,
        "restored tables must match"
    );

    let (status, discarded) = api.cancel_export(&organization_id, &project_id, &source_id).await;
    assert!(status.is_success(), "discarding a finished export should succeed: {discarded}");
    assert_eq!(discarded["state"], "idle");
    let (status, body) = api.export_download_token(&organization_id, &project_id, &source_id).await;
    assert_eq!(status.as_u16(), 409, "a discarded export cannot be downloaded: {body}");

    api.stop_running_endpoints(&organization_id, &project_id).await;
}

#[tokio::test]
async fn export_plain_sql_with_stopped_endpoint() {
    let _guard = exclusive().await;
    let api = api().await;

    let organization_id = api.create_organization("Plain Export Org").await;
    let project_id = api.create_project(&organization_id, "Plain Export Project").await;

    let source = api.create_branch(&organization_id, &project_id, "production", None).await;
    let source_id = source["id"].as_str().unwrap().to_string();
    let source_endpoint = api.start_endpoint(&organization_id, &project_id, &source_id).await;
    let source_client = connect(&source_endpoint).await;
    seed(&source_client, 50).await;
    create_extra_tables(&source_client).await;
    let expected_payload = checksum(&source_client).await;
    let expected_extra = extra_tables_checksum(&source_client).await;
    drop(source_client);

    api.stop_endpoint(&organization_id, &project_id, &source_id).await;
    assert_eq!(api.endpoint_status(&organization_id, &project_id, &source_id).await, "STOPPED");

    let (status, body) = api
        .start_export(
            &organization_id,
            &project_id,
            &source_id,
            json!({ "format": "plain", "database": "postgres", "no_owner": true, "no_privileges": true }),
        )
        .await;
    assert_eq!(status.as_u16(), 202, "export should be accepted: {body}");

    let (status, _) = wait_for_export(&api, &organization_id, &project_id, &source_id).await;
    assert_completed(&status);
    assert_eq!(
        api.endpoint_status(&organization_id, &project_id, &source_id).await,
        "RUNNING",
        "the export should have started the compute"
    );

    let (dump, headers) = download(&api, &organization_id, &project_id, &source_id).await;
    assert_eq!(headers["content-type"], "application/sql");
    assert!(
        headers["content-disposition"].to_str().unwrap().ends_with(".sql\""),
        "plain exports are .sql files"
    );
    let sql = String::from_utf8(dump.clone()).expect("a plain export is text");
    assert!(sql.contains("-- PostgreSQL database dump"), "not a pg_dump script");
    assert!(sql.contains("CREATE TABLE public.payload"), "payload table missing from the dump");
    assert!(sql.contains("CREATE TABLE reporting.daily"), "table in another schema missing from the dump");
    assert!(!sql.contains("OWNER TO"), "--no-owner should leave out ownership");
    assert!(!sql.contains("\nGRANT "), "--no-privileges should leave out grants");
    assert!(
        !sql.contains("CREATE EXTENSION IF NOT EXISTS neon ")
            && !sql.contains("neon_migration")
            && !sql.contains("health_check"),
        "Neon's internal objects must not be exported"
    );

    let target = api.create_branch(&organization_id, &project_id, "restored", None).await;
    let target_id = target["id"].as_str().unwrap().to_string();
    let target_endpoint = api.start_endpoint(&organization_id, &project_id, &target_id).await;
    restore_in_container(&dump, "plain", "v17", &target_endpoint);

    let target_client = connect(&target_endpoint).await;
    assert_eq!(checksum(&target_client).await, expected_payload, "restored payload must match");
    assert_eq!(
        extra_tables_checksum(&target_client).await,
        expected_extra,
        "restored tables must match"
    );

    api.stop_running_endpoints(&organization_id, &project_id).await;
}

#[tokio::test]
async fn export_can_be_cancelled() {
    let _guard = exclusive().await;
    let api = api().await;

    let organization_id = api.create_organization("Cancel Export Org").await;
    let project_id = api.create_project(&organization_id, "Cancel Export Project").await;

    let branch = api.create_branch(&organization_id, &project_id, "production", None).await;
    let branch_id = branch["id"].as_str().unwrap().to_string();
    let endpoint = api.start_endpoint(&organization_id, &project_id, &branch_id).await;
    let client = connect(&endpoint).await;
    seed(&client, 500).await;

    let (status, body) = api
        .start_export(&organization_id, &project_id, &branch_id, json!({ "format": "plain" }))
        .await;
    assert_eq!(status.as_u16(), 202, "export should be accepted: {body}");
    wait_for_export_state(&api, &organization_id, &project_id, &branch_id, "dumping").await;

    let (status, body) = api.cancel_export(&organization_id, &project_id, &branch_id).await;
    assert!(status.is_success(), "cancel should be accepted: {body}");

    let (status, _) = wait_for_export(&api, &organization_id, &project_id, &branch_id).await;
    assert_eq!(status["state"], "cancelled", "export should be cancelled: {status}");
    assert!(status["error"].is_null(), "a cancelled export is not an error: {status}");

    let (status, body) = api.export_download_token(&organization_id, &project_id, &branch_id).await;
    assert_eq!(status.as_u16(), 409, "a cancelled export cannot be downloaded: {body}");

    let (status, body) = api
        .start_export(&organization_id, &project_id, &branch_id, json!({ "format": "custom" }))
        .await;
    assert_eq!(status.as_u16(), 202, "a new export can start after a cancelled one: {body}");
    let (status, _) = wait_for_export(&api, &organization_id, &project_id, &branch_id).await;
    assert_completed(&status);

    let token = api.new_download_token(&organization_id, &project_id, &branch_id).await;
    api.delete_branch(&organization_id, &project_id, &branch_id).await;
    let response = api
        .download_export(&organization_id, &project_id, &branch_id, Some(&token))
        .await;
    assert!(
        response.status().is_client_error(),
        "the export of a deleted branch cannot be downloaded, got {}",
        response.status()
    );

    api.stop_running_endpoints(&organization_id, &project_id).await;
}

#[tokio::test]
async fn export_uses_pg_dump_matching_project_version() {
    let _guard = exclusive().await;
    let api = api().await;

    let organization_id = api.create_organization("Versioned Export Org").await;
    let project_id = api
        .create_project_with_version(&organization_id, "Postgres 14 Project", "V14")
        .await;

    let source = api.create_branch(&organization_id, &project_id, "production", None).await;
    let source_id = source["id"].as_str().unwrap().to_string();
    let source_endpoint = api.start_endpoint(&organization_id, &project_id, &source_id).await;
    let source_client = connect(&source_endpoint).await;
    seed(&source_client, 20).await;
    let expected_payload = checksum(&source_client).await;

    let (status, body) = api
        .start_export(&organization_id, &project_id, &source_id, json!({ "format": "plain" }))
        .await;
    assert_eq!(status.as_u16(), 202, "export should be accepted: {body}");
    assert_eq!(body["pg_version"], "V14");
    let (status, _) = wait_for_export(&api, &organization_id, &project_id, &source_id).await;
    assert_completed(&status);
    assert!(
        status["pg_dump_version"].as_str().is_some_and(|version| version.starts_with("14.")),
        "a Postgres 14 project must be exported with pg_dump 14: {status}"
    );

    let (dump, _) = download(&api, &organization_id, &project_id, &source_id).await;
    let sql = String::from_utf8(dump).expect("a plain export is text");
    assert!(sql.contains("-- Dumped from database version 14."), "unexpected server version in the dump");
    assert!(sql.contains("-- Dumped by pg_dump version 14."), "the dump was not made by pg_dump 14");
    assert!(
        !sql.contains("transaction_timeout"),
        "the dump must not use settings that Postgres 14 does not know"
    );

    let (status, body) = api
        .start_export(&organization_id, &project_id, &source_id, json!({ "format": "custom" }))
        .await;
    assert_eq!(status.as_u16(), 202, "export should be accepted: {body}");
    let (status, _) = wait_for_export(&api, &organization_id, &project_id, &source_id).await;
    assert_completed(&status);
    let (dump, _) = download(&api, &organization_id, &project_id, &source_id).await;

    let listing = pg_tool_in_container("v14", "pg_restore", &["--list"], &dump);
    assert!(
        listing.status.success(),
        "pg_restore 14 must be able to read the archive:\n{}",
        String::from_utf8_lossy(&listing.stderr)
    );
    assert!(
        String::from_utf8_lossy(&listing.stdout).contains("TABLE public payload"),
        "the archive should list the payload table"
    );

    let target = api.create_branch(&organization_id, &project_id, "restored", None).await;
    let target_id = target["id"].as_str().unwrap().to_string();
    let target_endpoint = api.start_endpoint(&organization_id, &project_id, &target_id).await;
    restore_in_container(&dump, "custom", "v14", &target_endpoint);
    let target_client = connect(&target_endpoint).await;
    assert_eq!(checksum(&target_client).await, expected_payload, "restored payload must match");

    api.stop_running_endpoints(&organization_id, &project_id).await;
}

#[tokio::test]
async fn export_rejects_invalid_and_unauthorized_requests() {
    let _guard = exclusive().await;
    let api = api().await;
    let outsider = api.other_user("e2e-outsider@example.com").await;

    let organization_id = api.create_organization("Export Security Org").await;
    let project_id = api.create_project(&organization_id, "Export Security Project").await;

    let branch = api.create_branch(&organization_id, &project_id, "production", None).await;
    let branch_id = branch["id"].as_str().unwrap().to_string();
    let other = api.create_branch(&organization_id, &project_id, "other", None).await;
    let other_id = other["id"].as_str().unwrap().to_string();
    let endpoint = api.start_endpoint(&organization_id, &project_id, &branch_id).await;
    let client = connect(&endpoint).await;
    seed(&client, 5).await;

    for body in [
        json!({ "format": "zip" }),
        json!({}),
        json!({ "format": "plain", "pg_dump_args": "--file=/etc/passwd" }),
        json!({ "format": "plain", "database": "" }),
        json!({ "format": "plain", "database": "dbname=postgres host=example.com" }),
        json!({ "format": "plain", "database": "postgresql://example.com/postgres" }),
    ] {
        let (status, response) = api.start_export(&organization_id, &project_id, &branch_id, body.clone()).await;
        assert!(
            status.is_client_error(),
            "request {body} should be rejected, got {status}: {response}"
        );
    }

    let (status, body) = api
        .start_export(&organization_id, &project_id, &branch_id, json!({ "format": "plain", "database": "missing" }))
        .await;
    assert_eq!(status.as_u16(), 202, "the database is checked by the export: {body}");
    let (status, _) = wait_for_export(&api, &organization_id, &project_id, &branch_id).await;
    assert_eq!(status["state"], "failed", "exporting a missing database should fail: {status}");
    assert!(
        status["error"].as_str().is_some_and(|error| error.contains("does not exist")),
        "the error should name the problem: {status}"
    );

    let (status, body) = outsider
        .start_export(&organization_id, &project_id, &branch_id, json!({ "format": "plain" }))
        .await;
    assert_eq!(status.as_u16(), 401, "a non-member must not start exports: {body}");

    let (status, body) = api
        .start_export(&organization_id, &project_id, &branch_id, json!({ "format": "plain" }))
        .await;
    assert_eq!(status.as_u16(), 202, "export should be accepted: {body}");
    let (status, _) = wait_for_export(&api, &organization_id, &project_id, &branch_id).await;
    assert_completed(&status);

    let (status, body) = outsider.export_status_raw(&organization_id, &project_id, &branch_id).await;
    assert_eq!(status.as_u16(), 401, "a non-member must not see the export: {body}");
    let (status, body) = outsider.export_download_token(&organization_id, &project_id, &branch_id).await;
    assert_eq!(status.as_u16(), 401, "a non-member must not get download tokens: {body}");
    let (status, body) = outsider.cancel_export(&organization_id, &project_id, &branch_id).await;
    assert_eq!(status.as_u16(), 401, "a non-member must not discard the export: {body}");

    for token in [None, Some("garbage"), Some(api.token())] {
        let response = api.download_export(&organization_id, &project_id, &branch_id, token).await;
        assert_eq!(
            response.status().as_u16(),
            401,
            "download with token {token:?} must be refused"
        );
    }

    let token = api.new_download_token(&organization_id, &project_id, &branch_id).await;
    assert_eq!(
        api.me_with_bearer(&token).await.as_u16(),
        401,
        "a download token must not work as an access token"
    );
    let response = api.download_export(&organization_id, &project_id, &other_id, Some(&token)).await;
    assert_eq!(response.status().as_u16(), 401, "a download token is bound to its branch");

    let response = api.download_export(&organization_id, &project_id, &branch_id, Some(&token)).await;
    assert_eq!(response.status().as_u16(), 200, "a valid token downloads the export");
    let _ = response.bytes().await;
    let response = api.download_export(&organization_id, &project_id, &branch_id, Some(&token)).await;
    assert_eq!(response.status().as_u16(), 401, "a download token can be used only once");

    let stale = api.new_download_token(&organization_id, &project_id, &branch_id).await;
    let (status, body) = api
        .start_export(&organization_id, &project_id, &branch_id, json!({ "format": "custom" }))
        .await;
    assert_eq!(status.as_u16(), 202, "export should be accepted: {body}");
    let response = api.download_export(&organization_id, &project_id, &branch_id, Some(&stale)).await;
    assert_eq!(response.status().as_u16(), 401, "a token of a replaced export must be refused");
    let (status, _) = wait_for_export(&api, &organization_id, &project_id, &branch_id).await;
    assert_completed(&status);
    let response = api.download_export(&organization_id, &project_id, &branch_id, Some(&stale)).await;
    assert_eq!(response.status().as_u16(), 401, "a token of a replaced export stays refused");

    api.stop_running_endpoints(&organization_id, &project_id).await;
}
