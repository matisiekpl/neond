use crate::mgmt::handler::AppState;
use crate::mgmt::repository::Repositories;
use crate::mgmt::repository::db::{init_pool, run_migrations};
use crate::mgmt::server::serve;
use crate::mgmt::service::Services;
use std::env::current_dir;
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

    let port: u16 = std::env::var("PORT")
        .expect("PORT must be set in .env")
        .parse()
        .expect("PORT must be a valid number");

    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env");

    let daemon_directory = current_dir()
        .expect("Failed to get current directory")
        .join("neon_daemon_data");

    let binaries_directory = tempfile::TempDir::new()?.keep();

    crate::preflight::check(daemon_directory.clone(), binaries_directory.clone())?;
    crate::unpacker::Unpacker::new(binaries_directory.clone())?.unpack()?;
    let mut daemon =
        crate::daemon::Daemon::new(daemon_directory.clone(), binaries_directory.clone());

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
    let services = Services::new(
        &repositories,
        Arc::new(pageserver_client),
        jwt_secret,
        daemon_directory,
    );
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

    serve(port, state).await?;
    Ok(())
}
