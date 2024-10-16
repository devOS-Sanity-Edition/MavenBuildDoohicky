use std::env;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::StatusCode;
use axum::http::uri::Parts;
use deadpool::managed::{Object, Pool};
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use crate::internal_error;

pub type DatabasePool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>, Object<AsyncDieselConnectionManager<AsyncPgConnection>>>;
pub type PooledConnection = Object<AsyncDieselConnectionManager<AsyncPgConnection>>;

#[derive(Clone)]
pub struct App {
    pub db: DatabasePool
}

impl App {
    pub fn new() -> Self {
        Self {
            db: {
                let database_url = env::var("DATABASE_URL")
                    .expect("DATABASE_URL must be set");

                let config =
                    AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
                Pool::builder(config).build().unwrap()
            }
        }
    }
}

pub struct DatabaseConnection(PooledConnection);

impl<S> FromRequestParts<S> for DatabaseConnection
where
    S: Send + Sync,
    DatabasePool: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool: DatabasePool = Pool::from_ref(state);

        let conn: PooledConnection = pool
            .get()
            .await
            .map_err(internal_error)?;

        Ok(Self(conn))
    }
}