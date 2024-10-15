use std::env;
use std::sync::Arc;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::post;
use axum_github_webhook_extract::{GithubEvent, GithubToken};
use serde::{Deserialize, Serialize};
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
        .route("/", post(root))
        .with_state(GithubToken(Arc::new(github_secret)))
        .layer(TraceLayer::new_for_http());

    let ip = env::var("APP_IP").unwrap_or("0.0.0.0".to_string());
    let port = env::var("APP_PORT").unwrap_or("3000".to_string());
    let address = format!("{}:{}", ip, port);

    log::info!("Listening on {}", address);

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// A git commit in specific payload types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Commit {
    pub id: String,
    pub message: String,
    pub distinct: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PushEventPayload {
    pub before: String,
    pub commits: Vec<Commit>,
}

async fn root(GithubEvent(e): GithubEvent<PushEventPayload>) -> impl IntoResponse {
    println!("Got event: {:?}", e.commits);
    e.before
}