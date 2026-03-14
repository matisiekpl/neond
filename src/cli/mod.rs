use crate::mgmt::handler::AppState;
use crate::mgmt::repository::db::{init_pool, run_migrations};
use crate::mgmt::repository::Repositories;
use crate::mgmt::server::serve;
use crate::mgmt::service::Services;

pub async fn run() {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().expect("Failed to load .env file");

    let port: u16 = std::env::var("PORT")
        .expect("PORT must be set in .env")
        .parse()
        .expect("PORT must be a valid number");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env");

    run_migrations(&database_url).await.expect("Failed to run migrations");

    init_pool(&database_url).await;
    let repositories = Repositories::new().await;
    let services = Services::new(&repositories, jwt_secret);
    let state = AppState { services };

    serve(port, state).await;
}
