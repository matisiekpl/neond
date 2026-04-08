use crate::mgmt::dto::config::Config;
use crate::mgmt::handler::AppState;
use crate::mgmt::repository::Repositories;
use crate::mgmt::repository::db::{init_pool, run_migrations};
use crate::mgmt::server::serve;
use crate::mgmt::service::Services;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing_panic::panic_hook;
use tracing_subscriber::EnvFilter;

pub async fn run() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    std::panic::set_hook(Box::new(panic_hook));
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    dotenvy::dotenv().expect("Failed to load .env file");

    let config = Config::new()?;
    crate::preflight::check(
        config.daemon_directory.clone(),
        config.binaries_directory.clone(),
        config.pg_proxy_port,
    )?;
    crate::unpacker::Unpacker::new(config.binaries_directory.clone())?.unpack()?;
    let mut daemon = crate::daemon::Daemon::new(config.clone());

    daemon.start().await?;
    let database_url = daemon.get_management_postgres_uri();

    run_migrations(&database_url)
        .await
        .expect("Failed to run migrations");

    init_pool(&database_url).await;

    let pageserver_http_client = reqwest::Client::new();
    let pageserver_api_token = config
        .component_auth
        .generate_token(neon_utils::auth::Scope::PageServerApi, None);
    let pageserver_client = neon_pageserver_client::mgmt_api::Client::new(
        pageserver_http_client,
        "http://127.0.0.1:1234".to_string(),
        Some(pageserver_api_token.as_str()),
    );

    let shutdown_token = CancellationToken::new();

    let repositories = Repositories::new().await;
    let services = Arc::new(Services::new(
        &repositories,
        Arc::new(pageserver_client),
        config.clone(),
        shutdown_token.clone(),
    ));
    let state = AppState {
        services: Arc::clone(&services),
    };

    let ctrlc_shutdown_token = shutdown_token.clone();
    ctrlc::set_handler(move || {
        ctrlc_shutdown_token.cancel();
    })?;

    services.endpoint().recover_running().await;

    let listen_services = Arc::clone(&services);
    tokio::spawn(async move {
        listen_services.endpoint().listen().await.unwrap();
    });

    serve(config.port, state).await?;

    services.endpoint().shutdown_all().await;
    daemon.stop()?;

    Ok(())
}