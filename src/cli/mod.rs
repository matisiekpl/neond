use crate::mgmt::dto::config::Config;
use crate::mgmt::handler::AppState;
use crate::mgmt::repository::Repositories;
use crate::mgmt::repository::db::{init_pool, run_migrations};
use crate::mgmt::server::serve;
use crate::mgmt::service::Services;
use std::process;
use std::sync::Arc;
use tokio::runtime::Handle;
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
    // TODO(matisiekpl): add authentication
    let pageserver_client = neon_pageserver_client::mgmt_api::Client::new(
        pageserver_http_client,
        "http://127.0.0.1:1234".to_string(),
        None,
    );

    let repositories = Repositories::new().await;
    let services = Services::new(&repositories, Arc::new(pageserver_client), config.clone());
    let state = AppState {
        services: Arc::new(services),
    };

    let shutdown_services = Arc::clone(&state.services);
    let rt_handle = Handle::current();
    ctrlc::set_handler(move || {
        rt_handle.block_on(shutdown_services.endpoint().shutdown_all());
        daemon.stop().unwrap();
        process::exit(0);
    })?;

    serve(config.port, state).await?;
    Ok(())
}
