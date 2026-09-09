use crate::common::{api, exclusive};

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
