use std::sync::Arc;
use axum::{Router, routing::post};
use tokio::net::TcpListener;

use crate::mgmt::handler::AppState;
use crate::mgmt::handler::user_handler;

pub async fn serve(port: u16, state: AppState) {
    let state = Arc::new(state);

    let app = Router::new()
        .route("/auth/login", post(user_handler::login))
        .route("/auth/register", post(user_handler::register))
        .with_state(state);

    let listener = TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("Failed to bind port");

    tracing::info!("Listening on 0.0.0.0:{}", port);

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
