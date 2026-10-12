use chrono::Utc;
use std::time::Duration;

use crate::common::{
    api, append, checksum, connect, exclusive, seed, wait_for_compaction, wait_for_ingest,
};

const SEED_MEGABYTES: i64 = 500;

#[tokio::test]
async fn reset_to_parent_restores_parent_state() {
    let _guard = exclusive().await;
    let api = api().await;

    let organization_id = api.create_organization("Reset Org").await;
    let project_id = api.create_project(&organization_id, "Reset Project").await;

    let parent = api
        .create_branch(&organization_id, &project_id, "production", None)
        .await;
    let parent_id = parent["id"].as_str().unwrap().to_string();
    let parent_endpoint = api.start_endpoint(&organization_id, &project_id, &parent_id).await;
    let parent_client = connect(&parent_endpoint).await;

    seed(&parent_client, SEED_MEGABYTES).await;
    wait_for_ingest(&api, &organization_id, &project_id, &parent_id, &parent_client).await;
    let parent_checksum_at_branch_point = checksum(&parent_client).await;

    let child = api
        .create_branch(&organization_id, &project_id, "child", Some(&parent_id))
        .await;
    let child_id = child["id"].as_str().unwrap().to_string();
    let child_endpoint = api.start_endpoint(&organization_id, &project_id, &child_id).await;
    let child_client = connect(&child_endpoint).await;

    assert_eq!(
        checksum(&child_client).await,
        parent_checksum_at_branch_point,
        "fresh branch should be identical to its parent"
    );

    child_client
        .batch_execute("DELETE FROM payload WHERE id % 3 = 0")
        .await
        .expect("failed to diverge child");
    append(&child_client, 50_000).await;
    assert_ne!(
        checksum(&child_client).await,
        parent_checksum_at_branch_point,
        "child should have diverged"
    );

    append(&parent_client, 50_000).await;
    wait_for_ingest(&api, &organization_id, &project_id, &parent_id, &parent_client).await;
    let parent_checksum_after_branch_point = checksum(&parent_client).await;

    let timeline_id_before = api
        .get_branch(&organization_id, &project_id, &child_id)
        .await["timeline_id"]
        .as_str()
        .unwrap()
        .to_string();

    let (status, reset) = api.reset(&organization_id, &project_id, &child_id).await;
    assert!(status.is_success(), "reset failed with {status}: {reset}");
    assert_ne!(
        reset["timeline_id"].as_str().unwrap(),
        timeline_id_before,
        "reset should swap the branch onto a new timeline"
    );
    assert_eq!(
        reset["parent_branch_id"].as_str(),
        Some(parent_id.as_str()),
        "reset must not change the parent"
    );
    let child_id = reset["id"].as_str().unwrap().to_string();
    assert_eq!(
        api.endpoint_status(&organization_id, &project_id, &child_id).await,
        "RUNNING",
        "a running endpoint should be restarted after reset"
    );

    let child_endpoint = api.endpoint(&organization_id, &project_id, &child_id).await;
    let child_client = connect(&child_endpoint).await;
    assert_eq!(
        checksum(&child_client).await,
        parent_checksum_after_branch_point,
        "reset branch must match the current state of its parent"
    );

    let (status, body) = api.reset(&organization_id, &project_id, &parent_id).await;
    assert!(
        status.is_client_error(),
        "resetting a branch without a parent should fail, got {status}: {body}"
    );

    api.create_branch(&organization_id, &project_id, "grandchild", Some(&child_id))
        .await;
    let (status, body) = api.reset(&organization_id, &project_id, &child_id).await;
    assert!(
        status.is_success(),
        "resetting a branch with children should succeed, got {status}: {body}"
    );
}

#[tokio::test]
async fn reset_to_parent_with_stopped_endpoint() {
    let _guard = exclusive().await;
    let api = api().await;

    let organization_id = api.create_organization("Reset Stopped Org").await;
    let project_id = api.create_project(&organization_id, "Reset Stopped Project").await;

    let parent = api
        .create_branch(&organization_id, &project_id, "production", None)
        .await;
    let parent_id = parent["id"].as_str().unwrap().to_string();
    let parent_endpoint = api.start_endpoint(&organization_id, &project_id, &parent_id).await;
    let parent_client = connect(&parent_endpoint).await;
    seed(&parent_client, SEED_MEGABYTES).await;
    wait_for_ingest(&api, &organization_id, &project_id, &parent_id, &parent_client).await;
    let parent_checksum = checksum(&parent_client).await;

    let child = api
        .create_branch(&organization_id, &project_id, "child", Some(&parent_id))
        .await;
    let child_id = child["id"].as_str().unwrap().to_string();
    let child_endpoint = api.start_endpoint(&organization_id, &project_id, &child_id).await;
    let child_client = connect(&child_endpoint).await;
    child_client
        .batch_execute("TRUNCATE payload")
        .await
        .expect("failed to diverge child");
    drop(child_client);
    api.stop_endpoint(&organization_id, &project_id, &child_id).await;

    let (status, reset) = api.reset(&organization_id, &project_id, &child_id).await;
    assert!(status.is_success(), "reset failed with {status}: {reset}");
    let child_id = reset["id"].as_str().unwrap().to_string();
    assert_ne!(
        api.endpoint_status(&organization_id, &project_id, &child_id).await,
        "RUNNING",
        "a stopped endpoint must stay stopped after reset"
    );

    let child_endpoint = api.start_endpoint(&organization_id, &project_id, &child_id).await;
    let child_client = connect(&child_endpoint).await;
    assert_eq!(checksum(&child_client).await, parent_checksum);
}

#[tokio::test]
async fn pitr_restores_point_in_time() {
    let _guard = exclusive().await;
    let api = api().await;

    let organization_id = api.create_organization("Pitr Org").await;
    let project_id = api.create_project(&organization_id, "Pitr Project").await;

    let branch = api
        .create_branch(&organization_id, &project_id, "production", None)
        .await;
    let branch_id = branch["id"].as_str().unwrap().to_string();
    let endpoint = api.start_endpoint(&organization_id, &project_id, &branch_id).await;
    let client = connect(&endpoint).await;

    seed(&client, SEED_MEGABYTES).await;
    wait_for_ingest(&api, &organization_id, &project_id, &branch_id, &client).await;
    let checksum_before_disaster = checksum(&client).await;
    let recovery_point = Utc::now();

    tokio::time::sleep(Duration::from_secs(10)).await;

    client
        .batch_execute("DROP TABLE payload")
        .await
        .expect("failed to drop payload table");
    drop(client);

    let timeline_id_before = api
        .get_branch(&organization_id, &project_id, &branch_id)
        .await["timeline_id"]
        .as_str()
        .unwrap()
        .to_string();

    let lsn = api
        .lsn_at(&organization_id, &project_id, &branch_id, recovery_point)
        .await;
    let (status, restored) = api.restore(&organization_id, &project_id, &branch_id, &lsn).await;
    assert!(status.is_success(), "restore failed with {status}: {restored}");
    assert_ne!(
        restored["timeline_id"].as_str().unwrap(),
        timeline_id_before,
        "restore should swap the branch onto a new timeline"
    );
    let branch_id = restored["id"].as_str().unwrap().to_string();
    assert_eq!(
        api.endpoint_status(&organization_id, &project_id, &branch_id).await,
        "RUNNING",
        "a running endpoint should be restarted after restore"
    );

    let endpoint = api.endpoint(&organization_id, &project_id, &branch_id).await;
    let client = connect(&endpoint).await;
    assert_eq!(
        checksum(&client).await,
        checksum_before_disaster,
        "restored branch must match the state at the recovery point"
    );

    let (status, body) = api
        .restore(&organization_id, &project_id, &branch_id, "not-an-lsn")
        .await;
    assert!(
        status.is_client_error(),
        "malformed lsn should be rejected, got {status}: {body}"
    );

    let (status, body) = api
        .restore(&organization_id, &project_id, &branch_id, "FFFFFFFF/FFFFFFFF")
        .await;
    assert!(
        status.is_client_error(),
        "out of range lsn should be rejected, got {status}: {body}"
    );
}

#[tokio::test]
async fn compaction_keeps_parent_and_child_data() {
    let _guard = exclusive().await;
    let api = api().await;

    let organization_id = api.create_organization("Compaction Org").await;
    let project_id = api.create_project(&organization_id, "Compaction Project").await;

    let parent = api
        .create_branch(&organization_id, &project_id, "production", None)
        .await;
    let parent_id = parent["id"].as_str().unwrap().to_string();
    let parent_endpoint = api.start_endpoint(&organization_id, &project_id, &parent_id).await;
    let parent_client = connect(&parent_endpoint).await;

    seed(&parent_client, SEED_MEGABYTES).await;
    wait_for_ingest(&api, &organization_id, &project_id, &parent_id, &parent_client).await;

    let child = api
        .create_branch(&organization_id, &project_id, "child", Some(&parent_id))
        .await;
    let child_id = child["id"].as_str().unwrap().to_string();
    let child_endpoint = api.start_endpoint(&organization_id, &project_id, &child_id).await;
    let child_client = connect(&child_endpoint).await;
    let child_checksum = checksum(&child_client).await;

    parent_client
        .batch_execute("UPDATE payload SET body = md5(body) WHERE id % 2 = 0")
        .await
        .expect("failed to rewrite parent rows");
    append(&parent_client, 50_000).await;
    wait_for_ingest(&api, &organization_id, &project_id, &parent_id, &parent_client).await;
    let parent_checksum = checksum(&parent_client).await;

    let (status, body) = api.compact(&organization_id, &project_id, &child_id).await;
    assert_eq!(status.as_u16(), 400, "compaction of a child branch should be rejected: {body}");

    let (status, body) = api.compact(&organization_id, &project_id, &parent_id).await;
    assert_eq!(status.as_u16(), 202, "compaction should be accepted: {body}");

    let status = wait_for_compaction(&api, &organization_id, &project_id, &parent_id).await;
    assert!(status["error"].is_null(), "compaction failed: {status}");
    assert!(
        status["size_after"].as_u64().unwrap() <= status["size_before"].as_u64().unwrap(),
        "compaction must not grow the branch: {status}"
    );

    api.stop_endpoint(&organization_id, &project_id, &parent_id).await;
    api.stop_endpoint(&organization_id, &project_id, &child_id).await;
    let parent_client = connect(&api.start_endpoint(&organization_id, &project_id, &parent_id).await).await;
    let child_client = connect(&api.start_endpoint(&organization_id, &project_id, &child_id).await).await;

    assert_eq!(
        checksum(&parent_client).await,
        parent_checksum,
        "compaction must not change parent data"
    );
    assert_eq!(
        checksum(&child_client).await,
        child_checksum,
        "compaction must not change data visible to the child branch"
    );
}

#[tokio::test]
async fn compaction_reclaims_hot_row_history() {
    let _guard = exclusive().await;
    let api = api().await;

    let organization_id = api.create_organization("Hot Rows Org").await;
    let project_id = api.create_project(&organization_id, "Hot Rows Project").await;
    let project_created = std::time::Instant::now();
    // Keep almost no history so that everything the load below writes falls under the GC cutoff
    // quickly, and flush it into layers right away.
    api.update_project(
        &organization_id,
        &project_id,
        serde_json::json!({
            "pitr_interval": "1m",
            "gc_horizon": 1048576,
            "gc_period": "10s",
            "checkpoint_timeout": "10s",
        }),
    )
    .await;

    let branch = api
        .create_branch(&organization_id, &project_id, "production", None)
        .await;
    let branch_id = branch["id"].as_str().unwrap().to_string();
    let endpoint = api.start_endpoint(&organization_id, &project_id, &branch_id).await;
    let client = connect(&endpoint).await;

    // A small table whose rows (and visibility map page) are rewritten over and over, like a hot
    // counters table in production.
    client
        .batch_execute(
            "CREATE TABLE hot (id int primary key, counter bigint not null, pad text not null); \
             INSERT INTO hot SELECT i, 0, repeat('x', 200) FROM generate_series(1, 2000) i;",
        )
        .await
        .expect("failed to create hot table");
    for _ in 0..20 {
        // Separate queries: a multi-statement query runs in one implicit transaction, where neither
        // COMMIT inside DO nor VACUUM is allowed.
        client
            .batch_execute(
                "DO $$ BEGIN FOR i IN 1..50 LOOP UPDATE hot SET counter = counter + 1; COMMIT; END LOOP; END $$",
            )
            .await
            .expect("failed to update hot rows");
        client.batch_execute("VACUUM hot").await.expect("failed to vacuum hot rows");
    }
    wait_for_ingest(&api, &organization_id, &project_id, &branch_id, &client).await;
    let total: i64 = client
        .query_one("SELECT sum(counter)::bigint FROM hot", &[])
        .await
        .expect("failed to read hot rows")
        .get(0);
    assert_eq!(total, 2000 * 20 * 50);

    // Let the load fall out of the PITR window. The pageserver also refuses to GC a tenant during
    // the LSN lease period after attaching it (10 minutes), and compaction needs a GC cutoff.
    tokio::time::sleep(Duration::from_secs(90)).await;
    let gc_allowed = project_created + Duration::from_secs(10 * 60 + 30);
    tokio::time::sleep(gc_allowed.saturating_duration_since(std::time::Instant::now())).await;

    let (status, body) = api.compact(&organization_id, &project_id, &branch_id).await;
    assert_eq!(status.as_u16(), 202, "compaction should be accepted: {body}");
    let (status, body) = api.compact(&organization_id, &project_id, &branch_id).await;
    assert_eq!(status.as_u16(), 409, "a second compaction must be rejected while one runs: {body}");

    let status = wait_for_compaction(&api, &organization_id, &project_id, &branch_id).await;
    println!("[compaction] {status}");
    assert!(status["error"].is_null(), "compaction failed: {status}");
    let size_before = status["size_before"].as_u64().unwrap();
    let size_after = status["size_after"].as_u64().unwrap();
    assert!(
        size_after < size_before,
        "compaction should reclaim the hot rows' history: {size_before} -> {size_after}"
    );

    drop(client);
    api.stop_endpoint(&organization_id, &project_id, &branch_id).await;
    let client = connect(&api.start_endpoint(&organization_id, &project_id, &branch_id).await).await;
    let total_after: i64 = client
        .query_one("SELECT sum(counter)::bigint FROM hot", &[])
        .await
        .expect("failed to read hot rows")
        .get(0);
    assert_eq!(total_after, total, "compaction must not change data");
}
