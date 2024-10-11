use std::env;
use std::sync::Arc;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::post;
use axum_github_webhook_extract::{GithubEvent, GithubToken};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    let github_secret = env::var("GITHUB_SECRET")
        .expect("Missing GITHUB_SECRET Environment Variable");

    let app = Router::new()
        .route("/", post(root))
        .with_state(GithubToken(Arc::new(github_secret)));

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("TcpListener bound to: {}", addr);
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