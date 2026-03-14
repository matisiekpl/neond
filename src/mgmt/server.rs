use std::sync::Arc;
use axum::{Router, routing::{post, get, delete, put}};
use tokio::net::TcpListener;

use crate::mgmt::handler::AppState;
use crate::mgmt::handler::{user, organization, project};

pub async fn serve(port: u16, state: AppState) {
    let state = Arc::new(state);

    let app = Router::new()
        .route("/auth/login", post(user::login))
        .route("/auth/register", post(user::register))
        .route("/organizations", post(organization::create).get(organization::list))
        .route("/organizations/{org_id}", get(organization::get).put(organization::update).delete(organization::delete))
        .route("/organizations/{org_id}/members", get(organization::list_members))
        .route("/organizations/{org_id}/members/{user_id}", post(organization::assign_member).delete(organization::revoke_member))
        .route("/organizations/{org_id}/projects", post(project::create).get(project::list))
        .route("/organizations/{org_id}/projects/{project_id}", get(project::get).put(project::update).delete(project::delete))
        .with_state(state);

    let listener = TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("Failed to bind port");

    tracing::info!("Listening on 0.0.0.0:{}", port);

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
