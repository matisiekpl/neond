use chrono::Utc;
use std::time::Duration;

use crate::common::{api, append, checksum, connect, exclusive, seed, wait_for_ingest};

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
