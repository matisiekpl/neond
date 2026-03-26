use reqwest::Client;
use serde_json::{json, Value};

const BASE_URL: &str = "http://127.0.0.1:3000";
const TEST_EMAIL: &str = "test@example.com";
const TEST_PASSWORD: &str = "test_password_123";
const TEST_NAME: &str = "Test User";

async fn register_or_login(client: &Client) -> String {
    println!("[auth] attempting to register user '{}' ({})", TEST_NAME, TEST_EMAIL);

    let register_res = client
        .post(format!("{BASE_URL}/auth/register"))
        .json(&json!({
            "name": TEST_NAME,
            "email": TEST_EMAIL,
            "password": TEST_PASSWORD
        }))
        .send()
        .await
        .expect("failed to send register request");

    let register_status = register_res.status();
    println!("[auth] register response status: {register_status}");

    if register_status.is_success() {
        let body: Value = register_res.json().await.expect("failed to parse register response");
        let token = body["token"]
            .as_str()
            .expect("token missing in register response")
            .to_string();
        println!("[auth] registration successful");
        return token;
    }

    // User already exists — fall back to login
    println!("[auth] registration returned {register_status}, falling back to login");

    let login_res = client
        .post(format!("{BASE_URL}/auth/login"))
        .json(&json!({
            "email": TEST_EMAIL,
            "password": TEST_PASSWORD
        }))
        .send()
        .await
        .expect("failed to send login request");

    let login_status = login_res.status();
    println!("[auth] login response status: {login_status}");

    assert!(
        login_status.is_success(),
        "login failed with status: {}",
        login_status
    );

    let body: Value = login_res.json().await.expect("failed to parse login response");
    let token = body["token"]
        .as_str()
        .expect("token missing in login response")
        .to_string();
    println!("[auth] login successful");
    token
}

#[tokio::test]
async fn test_full_mgmt_api_flow() {
    let client = Client::new();

    // Step 1: Register or login
    println!("[test] connecting to mgmt api at {BASE_URL}");
    let token = register_or_login(&client).await;
    println!("[test] authenticated, token: {}...", &token[..20.min(token.len())]);

    let auth_header = format!("Bearer {token}");

    // Step 2: Create organization
    println!("[test] creating organization 'Test Organization'");
    let org_res = client
        .post(format!("{BASE_URL}/organizations"))
        .header("Authorization", &auth_header)
        .json(&json!({ "name": "Test Organization" }))
        .send()
        .await
        .expect("failed to send create org request");

    let org_status = org_res.status();
    println!("[test] create org response status: {org_status}");
    assert!(
        org_status.is_success(),
        "create org failed with status: {}",
        org_status
    );

    let org: Value = org_res.json().await.expect("failed to parse org response");
    let org_id = org["id"].as_str().expect("org id missing");
    println!("[test] created organization: id={org_id}, name={}", org["name"].as_str().unwrap_or(""));

    // Step 3: Create project
    println!("[test] creating project 'Test Project' in org {org_id}");
    let project_res = client
        .post(format!("{BASE_URL}/organizations/{org_id}/projects"))
        .header("Authorization", &auth_header)
        .json(&json!({ "name": "Test Project" }))
        .send()
        .await
        .expect("failed to send create project request");

    let project_status = project_res.status();
    println!("[test] create project response status: {project_status}");
    assert!(
        project_status.is_success(),
        "create project failed with status: {}",
        project_status
    );

    let project: Value = project_res
        .json()
        .await
        .expect("failed to parse project response");
    let project_id = project["id"].as_str().expect("project id missing");
    println!(
        "[test] created project: id={project_id}, name={}, pg_version={}",
        project["name"].as_str().unwrap_or(""),
        project["pg_version"].as_str().unwrap_or("default")
    );

    // Step 4: Create branch
    println!("[test] creating branch 'main' in project {project_id}");
    let branch_res = client
        .post(format!(
            "{BASE_URL}/organizations/{org_id}/projects/{project_id}/branches"
        ))
        .header("Authorization", &auth_header)
        .json(&json!({ "name": "main" }))
        .send()
        .await
        .expect("failed to send create branch request");

    let branch_status = branch_res.status();
    println!("[test] create branch response status: {branch_status}");
    assert!(
        branch_status.is_success(),
        "create branch failed with status: {}",
        branch_status
    );

    let branch: Value = branch_res
        .json()
        .await
        .expect("failed to parse branch response");
    let branch_id = branch["id"].as_str().expect("branch id missing");
    println!(
        "[test] created branch: id={branch_id}, name={}, slug={}, timeline_id={}",
        branch["name"].as_str().unwrap_or(""),
        branch["slug"].as_str().unwrap_or(""),
        branch["timeline_id"].as_str().unwrap_or("")
    );

    // Step 5: Start endpoint
    println!("[test] starting endpoint for branch {branch_id}");
    let endpoint_res = client
        .post(format!(
            "{BASE_URL}/organizations/{org_id}/projects/{project_id}/branches/{branch_id}/endpoint"
        ))
        .header("Authorization", &auth_header)
        .send()
        .await
        .expect("failed to send start endpoint request");

    let endpoint_status = endpoint_res.status();
    println!("[test] start endpoint response status: {endpoint_status}");
    assert!(
        endpoint_status.is_success(),
        "start endpoint failed with status: {}",
        endpoint_status
    );

    let endpoint: Value = endpoint_res
        .json()
        .await
        .expect("failed to parse endpoint response");

    println!(
        "[test] endpoint started: branch_id={}, status={}, port={}",
        endpoint["branch_id"].as_str().unwrap_or(""),
        endpoint["status"].as_str().unwrap_or("unknown"),
        endpoint["port"].as_u64().unwrap_or(0)
    );
    println!("[test] full flow completed successfully");
    println!();
    println!("[cmd] curl to list branches:");
    println!(
        "  curl -s -X GET {BASE_URL}/organizations/{org_id}/projects/{project_id}/branches -H 'Authorization: {auth_header}'"
    );
    println!("[cmd] curl to start endpoint:");
    println!(
        "  curl -s -X POST {BASE_URL}/organizations/{org_id}/projects/{project_id}/branches/{branch_id}/endpoint -H 'Authorization: {auth_header}'"
    );
    println!("[cmd] curl to stop endpoint:");
    println!(
        "  curl -s -X DELETE {BASE_URL}/organizations/{org_id}/projects/{project_id}/branches/{branch_id}/endpoint -H 'Authorization: {auth_header}'"
    );

    assert_eq!(
        endpoint["branch_id"].as_str().expect("branch_id missing in endpoint response"),
        branch_id,
        "endpoint branch_id mismatch"
    );
    assert!(
        endpoint["port"].as_u64().unwrap_or(0) > 0,
        "endpoint port should be non-zero"
    );
}
