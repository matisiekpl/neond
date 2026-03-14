use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;

pub type DbPool = Pool<AsyncPgConnection>;

pub async fn create_pool(database_url: &str) -> DbPool {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create DB pool")
}
