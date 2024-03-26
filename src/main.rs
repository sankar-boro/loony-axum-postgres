mod auth;
mod blog;
mod book;
mod file;
mod route;

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use bb8::Pool;
use bb8_postgres::{bb8, PostgresConnectionManager};
use tokio_postgres::NoTls;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub pg_pool: Pool<PostgresConnectionManager<NoTls>>,
}

async fn create_connection() -> AppState {
    let pg_host = std::env::var("PG_HOST").unwrap();
    let pg_user = std::env::var("PG_USER").unwrap();
    let pg_dbname = std::env::var("PG_DBNAME").unwrap();
    let pg_password = std::env::var("PB_PASSWORD").unwrap();

    // set up connection pool
    let pg_manager = PostgresConnectionManager::new_from_stringlike(
        format!(
            "host={} user={} dbname={} password={}",
            pg_host, pg_user, pg_dbname, pg_password
        ),
        NoTls,
    )
    .unwrap();
    let pg_pool = Pool::builder().build(pg_manager).await.unwrap();

    return AppState {
        pg_pool,
        // redis_pool
    };
}

#[tokio::main]
async fn main() {
    let host = std::env::var("HOST").unwrap();
    let port = std::env::var("PORT").unwrap();
    let allow_origin = std::env::var("ALLOW_ORIGIN").unwrap();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_tokio_postgres=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let connection = create_connection().await;

    let cors = CorsLayer::new()
        .allow_origin(allow_origin.parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let router = route::create_router(connection, cors).await;

    // run it
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}"))
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router).await.unwrap();
}
