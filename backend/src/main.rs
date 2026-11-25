use axum::{
    routing::get,
    Router,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // Set env variable for port
    let port = match std::env::var("PORT") {
        Ok(p) => p,
        Err(_) => "3000".to_string(),
    };

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
}