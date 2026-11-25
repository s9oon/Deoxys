use axum::{routing::get, Json, Router};
use shared::Hello;

async fn hello() -> Json<Hello> {
    Json(Hello { message: "Hello, World!".to_string() })
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // build our application with a single route
    let app = Router::new().route("/", get(hello));

    // Set env variable for port
    let port = match std::env::var("PORT") {
        Ok(p) => p,
        Err(_) => "3000".to_string(),
    };

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    axum::serve(listener, app).await?;
    Ok(())
}