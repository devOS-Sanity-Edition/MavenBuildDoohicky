use std::env;
use std::sync::Arc;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use axum_github_webhook_extract::{GithubEvent, GithubToken};
use serde::Deserialize;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    let level = if cfg!(debug_assertions) {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();

    dotenvy::dotenv().ok();

    let github_secret = env::var("GITHUB_SECRET")
        .expect("Missing GITHUB_SECRET Environment Variable");

    let app = Router::new()
        .route("/", get(root))
        .with_state(GithubToken(Arc::new(github_secret)))
        .layer(TraceLayer::new_for_http());

    let ip = env::var("APP_IP").unwrap_or("0.0.0.0".to_string());
    let port = env::var("APP_PORT").unwrap_or("3000".to_string());
    let address = format!("{}:{}", ip, port);

    log::info!("Listening on {}", address);

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize)]
struct Event {
    action: String,
}

async fn root(GithubEvent(e): GithubEvent<Event>) -> impl IntoResponse {
    e.action
}