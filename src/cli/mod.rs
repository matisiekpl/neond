use crate::daemon::backup::BackupService;
use crate::mgmt::dto::config::Config;
use crate::mgmt::dto::error::{AppError, Result};
use crate::mgmt::handler::AppState;
use crate::mgmt::repository::Repositories;
use crate::mgmt::repository::db::{init_pool, run_migrations};
use crate::mgmt::server::serve;
use crate::mgmt::service::Services;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing_panic::panic_hook;
use tracing_subscriber::EnvFilter;

pub async fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    std::panic::set_hook(Box::new(panic_hook));
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| AppError::CryptoProviderInitFailed {
            reason: "default provider already installed".to_string(),
        })?;

    if let Err(err) = dotenvy::dotenv() {
        tracing::warn!("Failed to load .env file: {err}");
    }

    let config = Config::new()?;
    crate::preflight::check(
        config.daemon_directory.clone(),
        config.neon_binaries_directory.clone(),
        config.pg_install_directory.clone(),
        config.pg_proxy_port,
        config.remote_storage_config.clone(),
    )
    .await
    .map_err(|error| AppError::ApplicationStartupFailed {
        reason: format!("preflight check: {}", error),
    })?;
    let shutdown_token = CancellationToken::new();

    let backup_service = Arc::new(
        BackupService::new(config.clone(), shutdown_token.clone()).await?,
    );

    let mut daemon = crate::daemon::Daemon::new(config.clone(), Arc::clone(&backup_service))?;

    daemon.start().await?;

    let database_url = daemon.get_management_postgres_uri();

    run_migrations(&database_url).await?;

    init_pool(&database_url).await?;

    Arc::clone(&backup_service)
        .start_periodic(daemon.backed_up_databases())
        .await;

    let pageserver_http_client = reqwest::Client::new();
    let pageserver_api_token = config
        .component_auth
        .generate_token(neon_utils::auth::Scope::PageServerApi, None)?;
    let pageserver_client = neon_pageserver_client::mgmt_api::Client::new(
        pageserver_http_client,
        "http://127.0.0.1:1234".to_string(),
        Some(pageserver_api_token.as_str()),
    );

    let safekeeper_http_client = reqwest::Client::new();
    let safekeeper_api_token = config
        .component_auth
        .generate_token(neon_utils::auth::Scope::SafekeeperData, None)?;
    let safekeeper_client = neon_safekeeper_client::mgmt_api::Client::new(
        safekeeper_http_client,
        "http://127.0.0.1:7676".to_string(),
        Some(neon_utils::logging::SecretString::from(safekeeper_api_token)),
    );

    let repositories = Repositories::new().await?;
    let services = Arc::new(Services::new(
        &repositories,
        Arc::new(pageserver_client),
        Arc::new(safekeeper_client),
        config.clone(),
        shutdown_token.clone(),
    ));
    let state = AppState {
        services: Arc::clone(&services),
    };

    let ctrlc_shutdown_token = shutdown_token.clone();
    ctrlc::set_handler(move || {
        ctrlc_shutdown_token.cancel();
    })
    .map_err(|error| AppError::ApplicationStartupFailed {
        reason: format!("ctrlc handler: {}", error),
    })?;

    services.endpoint().recover_running().await;

    let listen_services = Arc::clone(&services);
    tokio::spawn(async move {
        if let Err(error) = listen_services.endpoint().listen().await {
            tracing::error!("Endpoint listen task failed: {}", error);
        }
    });

    let metric_service = Arc::clone(services.metric());
    tokio::spawn(
        Arc::clone(&metric_service).run_collection_loop(shutdown_token.clone()),
    );
    tokio::spawn(metric_service.run_cleanup_loop(shutdown_token.clone()));

    serve(config.port, state)
        .await
        .map_err(|error| AppError::ApplicationStartupFailed {
            reason: format!("server: {}", error),
        })?;

    services.endpoint().shutdown_all().await;
    backup_service.stop_periodic().await;
    backup_service.final_sync(&daemon.backed_up_databases()).await;
    daemon.stop()?;

    Ok(())
}
