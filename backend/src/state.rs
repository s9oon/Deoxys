use sqlx::PgPool;

pub struct AppState {
    pub database: PgPool,
}

impl AppState {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        Ok(Self { database: PgPool::connect(database_url).await? })
    }
}