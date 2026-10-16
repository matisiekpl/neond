use reqwest::StatusCode;
use serde_json::json;

use crate::common::{api, connect, exclusive};

#[tokio::test]
async fn full_mgmt_api_flow() {
    let _guard = exclusive().await;
    let api = api().await;

    let organization_id = api.create_organization("Test Organization").await;
    let project_id = api.create_project(&organization_id, "Test Project").await;
    let branch = api
        .create_branch(&organization_id, &project_id, "main", None)
        .await;
    let branch_id = branch["id"].as_str().expect("branch id missing").to_string();

    let endpoint = api.start_endpoint(&organization_id, &project_id, &branch_id).await;
    assert_eq!(
        endpoint["branch_id"].as_str().expect("branch_id missing"),
        branch_id
    );
    assert!(
        endpoint["port"].as_u64().unwrap_or(0) > 0,
        "endpoint port should be non-zero"
    );

    let branches = api.get_branch(&organization_id, &project_id, &branch_id).await;
    assert_eq!(branches["slug"].as_str().map(|slug| slug.is_empty()), Some(false));

    api.stop_endpoint(&organization_id, &project_id, &branch_id).await;
}

#[tokio::test]
async fn compute_settings_apply_to_postgres_conf() {
    let _guard = exclusive().await;
    let api = api().await;

    let organization_id = api.create_organization("Compute Settings Organization").await;
    let project_id = api.create_project(&organization_id, "Compute Settings Project").await;
    let branch = api
        .create_branch(&organization_id, &project_id, "main", None)
        .await;
    let branch_id = branch["id"].as_str().expect("branch id missing").to_string();

    let (status, _) = api
        .update_compute_settings(&organization_id, &project_id, &branch_id, json!({ "shared_buffers_mb": 1 }))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, settings) = api
        .update_compute_settings(
            &organization_id,
            &project_id,
            &branch_id,
            json!({ "shared_buffers_mb": 256, "work_mem_mb": 12 }),
        )
        .await;
    assert!(status.is_success(), "update failed with {status}: {settings}");
    assert_eq!(settings["effective"]["effective_cache_size_mb"], 1024);
    assert_eq!(settings["effective"]["work_mem_mb"], 12);
    assert!(settings["memory"]["host_total_mb"].as_i64().unwrap_or(0) > 0);

    let endpoint = api.start_endpoint(&organization_id, &project_id, &branch_id).await;
    let client = connect(&endpoint).await;
    for (name, expected) in [
        ("shared_buffers", "256MB"),
        ("effective_cache_size", "1GB"),
        ("work_mem", "12MB"),
        ("maintenance_work_mem", "256MB"),
    ] {
        let row = client
            .query_one(format!("SHOW {name}").as_str(), &[])
            .await
            .expect("SHOW failed");
        assert_eq!(row.get::<_, String>(0), expected, "{name}");
    }

    api.stop_endpoint(&organization_id, &project_id, &branch_id).await;
}
