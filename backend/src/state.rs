// Shared data every handler can reach through axum's `State` extractor.
use sqlx::PgPool;
pub struct AppState {
    pub database: PgPool,
}
