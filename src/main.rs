use std::env;
use std::sync::Arc;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use axum_github_webhook_extract::{GithubEvent, GithubToken};
use serde::Deserialize;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    let github_secret = env::var("GITHUB_SECRET")
        .expect("Missing GITHUB_SECRET Environment Variable");

    let app = Router::new()
        .route("/", get(root))
        .with_state(GithubToken(Arc::new(github_secret)));

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("TcpListener bound to: {}", addr);
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize)]
struct Event {
    action: String,
}

async fn root(GithubEvent(e): GithubEvent<Event>) -> impl IntoResponse {
    e.action
}