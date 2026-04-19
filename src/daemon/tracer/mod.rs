use axum::{Router, http::StatusCode, response::IntoResponse, routing::any};
use tokio::sync::oneshot;

pub(crate) struct Tracer {
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Tracer {
    pub fn new() -> Self {
        Tracer { shutdown_tx: None }
    }

    pub fn start(&mut self) {
        let (tx, rx) = oneshot::channel();
        self.shutdown_tx = Some(tx);

        tokio::spawn(async move {
            let app = Router::new().route("/{*path}", any(Self::handler));
            let listener = match tokio::net::TcpListener::bind(("127.0.0.1", 4318)).await {
                Ok(listener) => listener,
                Err(error) => {
                    tracing::error!("Tracer startup failed: bind error: {}", error);
                    return;
                }
            };

            tracing::info!("Tracer listening on 127.0.0.1:{}", 4318);

            if let Err(error) = axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = rx.await;
                    tracing::info!("Shutting down tracer...");
                })
                .await
            {
                tracing::error!("Tracer serve error: {}", error);
            }
        });
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }

    async fn handler() -> impl IntoResponse {
        (StatusCode::OK, "{}")
    }
}
