use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use crate::mgmt::handler::AppState;
use crate::mgmt::handler::{branch, daemon, endpoint, organization, project, user};

pub async fn serve(port: u16, state: AppState) -> Result<(), anyhow::Error> {
    // TODO(matisiekpl): add PITR restoration
    // TODO(matisiekpl): display tenant size
    // TODO(matisiekpl): add ability to detach ancestor
    // TODO(matisiekpl): make Dockerfile and docker compose
    // TODO(matisiekpl): add ability to see compute endpoint logs
    // TODO(matisiekpl): add global daemon event log
    // TODO(matisiekpl): optimize postgres parameters
    // TODO(matisiekpl): handle unwrap and expect
    let shutdown_token = state.services.daemon().shutdown_token();
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
        .route("/daemon", get(daemon::get))
        .route(
            "/daemon/shutdown",
            post(daemon::shutdown).delete(daemon::cancel_shutdown),
        )
        .with_state(state);

    let app = Router::new()
        .nest("/api", api)
        .layer(CorsLayer::permissive());

    #[cfg(not(debug_assertions))]
    let app = app.fallback(crate::mgmt::frontend::static_handler);

    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!("Listening on 0.0.0.0:{}", port);

    axum::serve(listener, app)
        .with_graceful_shutdown(async move { shutdown_token.cancelled().await })
        .await?;
    Ok(())
}
