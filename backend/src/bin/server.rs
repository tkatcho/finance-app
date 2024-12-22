use axum::{
    routing::get,
    Router,
};
use serde_json::json;
use tower::ServiceBuilder;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Build the Axum app
    let app = Router::new()
        .route("/", get(root))
        .layer(ServiceBuilder::new()); // Middleware can be added here later
    
    // Define the address where the server will run
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);

    // Run the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// This is your basic route handler that will return JSON
async fn root() -> axum::Json<serde_json::Value> {
    axum::Json(json!({
        "message": "Welcome to your Finance App Backend!"
    }))
}