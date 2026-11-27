use sqlx::PgPool;

/// Shared state every handler can access (wrapped in Arc in main.rs)
pub struct AppState {
    /// Pool of database connections, reused across requests
    pub database: PgPool,
}

impl AppState {
    /// Connects to Postgres and builds the state. Fails if the database is unreachable.
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        Ok(Self { database: PgPool::connect(database_url).await? })
    }
}