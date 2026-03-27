use axum::{
    Router,
    routing::{delete, get, post, put},
};
use neon_utils::shard::TenantShardId;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use crate::mgmt::handler::AppState;
use crate::mgmt::handler::{branch, endpoint, organization, project, user};

pub async fn serve(port: u16, state: AppState) -> Result<(), anyhow::Error> {
    // TODO(matisiekpl): integrate React hosting into app
    // TODO(matisiekpl): add PITR restoration
    // TODO(matisiekpl): add configurable PITR horizon
    // TODO(matisiekpl): add daemon settings - list mappings, active branches, durability of them
    // TODO(matisiekpl): add button to shutdown daemon
    // TODO(matisiekpl): in daemon settings show used remote storage
    // TODO(matisiekpl): display tenant size
    // TODO(matisiekpl): add ability to detach ancestor
    // TODO(matisiekpl): add option to configure gc_period, gc_horizon, pitr_interval, checkpoint_distance, checkpoint_timeout
    let state = Arc::new(state);

    let api = Router::new()
        .route("/auth/login", post(user::login))
        .route("/auth/register", post(user::register))
        .route("/auth/me", get(user::me))
        .route(
            "/organizations",
            post(organization::create).get(organization::list),
        )
        .route(
            "/organizations/{org_id}",
            get(organization::get)
                .put(organization::update)
                .delete(organization::delete),
        )
        .route(
            "/organizations/{org_id}/members",
            get(organization::list_members).post(organization::assign_member_by_email),
        )
        .route(
            "/organizations/{org_id}/members/{user_id}",
            post(organization::assign_member).delete(organization::revoke_member),
        )
        .route(
            "/organizations/{org_id}/projects",
            post(project::create).get(project::list),
        )
        .route(
            "/organizations/{org_id}/projects/{project_id}",
            get(project::get)
                .put(project::update)
                .delete(project::delete),
        )
        .route(
            "/organizations/{org_id}/projects/{project_id}/branches",
            post(branch::create).get(branch::list),
        )
        .route(
            "/organizations/{org_id}/projects/{project_id}/branches/{branch_id}",
            put(branch::update).delete(branch::delete),
        )
        .route(
            "/organizations/{org_id}/projects/{project_id}/branches/{branch_id}/endpoint",
            post(endpoint::start)
                .delete(endpoint::stop)
                .get(endpoint::status),
        )
        .with_state(state);

    let app = Router::new()
        .nest("/api", api)
        .layer(CorsLayer::permissive());

    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!("Listening on 0.0.0.0:{}", port);

    axum::serve(listener, app).await?;
    Ok(())
}
