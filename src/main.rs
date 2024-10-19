mod app;
mod schema;

use crate::app::{App, DatabaseConnection};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Router;
use axum_github_webhook_extract::{GithubEvent, GithubToken};
use diesel::row::NamedRow;
use diesel_async::RunQueryDsl;
use diesel_async_migrations::{embed_migrations, EmbeddedMigrations};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

#[tokio::main]
async fn main() {
    {
        let level = if cfg!(debug_assertions) {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        };

        tracing_subscriber::fmt()
            .with_max_level(level)
            .init();
    }

    dotenvy::dotenv().ok();

    let app = App::new();

    {
        let mut conn = app.db.get().await.unwrap();
        MIGRATIONS.run_pending_migrations(&mut conn).await.unwrap();
    }

    let github_secret = env::var("GITHUB_SECRET")
        .expect("Missing GITHUB_SECRET Environment Variable");

    let router = Router::new()
        .with_state(app)
        //.route("/", post(root))
        .with_state(GithubToken(Arc::new(github_secret)))
        .layer(TraceLayer::new_for_http());

    let ip = env::var("APP_IP").unwrap_or("0.0.0.0".to_string());
    let port = env::var("APP_PORT").unwrap_or("3000".to_string());
    let address = format!("{}:{}", ip, port);

    log::info!("Listening on {}", address);

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

/// Utility function for mapping any error into a `500 Internal Server Error` response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}