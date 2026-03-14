use diesel::{ConnectionError, ConnectionResult};
use diesel_async::{AsyncMigrationHarness, AsyncPgConnection};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::ManagerConfig;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use futures_util::FutureExt;
use rustls::ClientConfig;
use rustls_platform_verifier::ConfigVerifierExt;
use std::time::Duration;
use tokio::sync::OnceCell;

pub type DbPool = Pool<AsyncPgConnection>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

static POOL: OnceCell<DbPool> = OnceCell::const_new();

fn establish_tls_connection(config: &str) -> futures_util::future::BoxFuture<'_, ConnectionResult<AsyncPgConnection>> {
    let fut = async {
        let rustls_config = ClientConfig::with_platform_verifier();
        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config);

        // TODO(matisiekpl): simplify this in future
        let mut config_str = config.to_string();
        config_str = config_str.replace("sslmode=require", "")
                                .replace("sslmode=require&", "")
                                .replace("&sslmode=require", "");
        config_str = config_str.replace("channel_binding=require", "")
                                .replace("channel_binding=require&", "")
                                .replace("&channel_binding=require", "");
        config_str = config_str.replace("?&", "?")
                                .trim_end_matches('?')
                                .to_string();

        let (client, conn) = tokio_postgres::connect(&config_str, tls)
            .await
            .map_err(|e| {
                tracing::error!("TLS connection error: {}", e);
                ConnectionError::BadConnection(e.to_string())
            })?;

        AsyncPgConnection::try_from_client_and_connection(client, conn)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create AsyncPgConnection: {}", e);
                ConnectionError::BadConnection(e.to_string())
            })
    };
    fut.boxed()
}

pub async fn init_pool(database_url: &str) {
    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_tls_connection);

    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(database_url.to_string(), config);
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
    let async_connection = establish_tls_connection(database_url).await?;
    let mut harness = AsyncMigrationHarness::new(async_connection);
    harness.run_pending_migrations(MIGRATIONS)?;
    tracing::info!("Migrations completed successfully");
    Ok(())
}
