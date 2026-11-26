mod state;
mod websocket;

use std::sync::Arc;
use axum::{http::StatusCode, routing::{get, any}, Json, Router, extract::State};
use shared::Hello;
use state::AppState;

async fn hello() -> Json<Hello> {
    Json(Hello { message: "Hello, World!".to_string() })
}

async fn health(State(state): State<Arc<AppState>>) -> StatusCode {
    match sqlx::query("SELECT 1").execute(&state.database).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read config from env at startup
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let database_url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL not set")?;

    let state = Arc::new(AppState::connect(&database_url).await?);

    let app = Router::new()
        .route("/", get(hello))
        .route("/health", get(health))
        .route("/ws", any(websocket::handler))
        .with_state(state);

    // Listen on all interfaces so it's reachable inside a container
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
