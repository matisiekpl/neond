use crate::mgmt::dto::error::{AppError, Result};
use diesel::PgConnection;
use diesel_async::AsyncMigrationHarness;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use diesel_migrations::MigrationHarness;
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
use std::time::Duration;
use tokio::sync::OnceCell;

pub type DbPool = Pool<AsyncPgConnection>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

static POOL: OnceCell<DbPool> = OnceCell::const_new();

pub async fn init_pool(database_url: &str) -> Result<()> {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(10)
        .min_idle(Some(5))
        .max_lifetime(Some(Duration::from_secs(60 * 60 * 24)))
        .idle_timeout(Some(Duration::from_secs(60 * 2)))
        .build(manager)
        .await
        .map_err(|error| AppError::DatabasePoolInitFailed {
            reason: error.to_string(),
        })?;
    POOL.get_or_init(|| async { pool }).await;
    Ok(())
}

pub async fn get_pool() -> Result<&'static DbPool> {
    POOL.get().ok_or(AppError::DatabasePoolNotInitialized)
}

pub async fn run_migrations(database_url: &str) -> Result<()> {
    let conn = AsyncPgConnection::establish(database_url)
        .await
        .map_err(|error| AppError::DatabaseMigrationsFailed {
            reason: error.to_string(),
        })?;
    let mut harness = AsyncMigrationHarness::new(conn);
    harness
        .run_pending_migrations(MIGRATIONS)
        .map_err(|error| AppError::DatabaseMigrationsFailed {
            reason: error.to_string(),
        })?;
    tracing::info!("Migrations completed successfully");
    Ok(())
}