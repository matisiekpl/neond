use crate::mgmt::handler::AppState;
use crate::mgmt::repository::Repositories;
use crate::mgmt::repository::db::{init_pool, run_migrations};
use crate::mgmt::server::serve;
use crate::mgmt::service::Services;
use tracing_subscriber::EnvFilter;

pub async fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    dotenvy::dotenv().expect("Failed to load .env file");

    let port: u16 = std::env::var("PORT")
        .expect("PORT must be set in .env")
        .parse()
        .expect("PORT must be a valid number");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env");

    run_migrations(&database_url)
        .await
        .expect("Failed to run migrations");

    init_pool(&database_url).await;
    let repositories = Repositories::new().await;
    let services = Services::new(&repositories, jwt_secret);
    let state = AppState { services };

    serve(port, state).await;
}
