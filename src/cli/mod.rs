use std::sync::Arc;

use crate::mgmt::handler::AppState;
use crate::mgmt::repository::db::{create_pool, run_migrations};
use crate::mgmt::repository::user_repository::UserRepository;
use crate::mgmt::server::serve;
use crate::mgmt::service::user_service::UserService;

pub async fn run() {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().expect("Failed to load .env file");

    let port: u16 = std::env::var("PORT")
        .expect("PORT must be set in .env")
        .parse()
        .expect("PORT must be a valid number");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env");

    run_migrations(&database_url);

    let pool = create_pool(&database_url).await;
    let user_repo = Arc::new(UserRepository::new(pool));
    let user_service = UserService::new(user_repo, jwt_secret);
    let state = AppState { user_service };

    serve(port, state).await;
}
