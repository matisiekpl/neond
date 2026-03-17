use diesel::PgConnection;
use diesel_async::AsyncMigrationHarness;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
use std::time::Duration;
use diesel_migrations::MigrationHarness;
use tokio::sync::OnceCell;

pub type DbPool = Pool<AsyncPgConnection>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

static POOL: OnceCell<DbPool> = OnceCell::const_new();

pub async fn init_pool(database_url: &str) {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(10)
        .min_idle(Some(5))
        .max_lifetime(Some(Duration::from_secs(60 * 60 * 24)))
        .idle_timeout(Some(Duration::from_secs(60 * 2)))
        .build(manager)
        .await
        .expect("Failed to create DB pool");
    POOL.get_or_init(|| async { pool }).await;
}

pub async fn get_pool() -> &'static DbPool {
    POOL.get()
        .expect("Pool not initialized. Call init_pool first")
}

pub async fn run_migrations(database_url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let conn = AsyncPgConnection::establish(database_url).await?;
    let mut harness = AsyncMigrationHarness::new(conn);
    harness.run_pending_migrations(MIGRATIONS)?;
    tracing::info!("Migrations completed successfully");
    Ok(())
}