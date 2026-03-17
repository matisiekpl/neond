use crate::mgmt::handler::AppState;
use crate::mgmt::repository::Repositories;
use crate::mgmt::repository::db::{init_pool, run_migrations};
use crate::mgmt::server::serve;
use crate::mgmt::service::Services;
use std::env::current_dir;
use std::process;
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

    crate::preflight::check(daemon_directory.clone())?;
    crate::unpacker::Unpacker::new(daemon_directory.clone())?.unpack()?;
    let mut daemon = crate::daemon::Daemon::new(daemon_directory.clone());

    daemon.start()?;
    let database_url = daemon.get_management_postgres_uri();
    ctrlc::set_handler(move || {
        daemon.stop().unwrap();
        process::exit(0);
    })?;

    run_migrations(&database_url)
        .await
        .expect("Failed to run migrations");

    init_pool(&database_url).await;
    let repositories = Repositories::new().await;
    let services = Services::new(&repositories, jwt_secret);
    let state = AppState { services };

    serve(port, state).await?;
    Ok(())
}
