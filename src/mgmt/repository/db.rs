use diesel::{Connection, PgConnection};
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub type DbPool = Pool<AsyncPgConnection>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub async fn create_pool(database_url: &str) -> DbPool {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create DB pool")
}

pub fn run_migrations(database_url: &str) {
    let mut conn = PgConnection::establish(database_url)
        .expect("Failed to connect to database for migrations");
    let applied = conn
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
    if applied.is_empty() {
        tracing::info!("No pending migrations");
    } else {
        tracing::info!("{} migration(s) applied", applied.len());
    }
}
