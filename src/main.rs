mod route;
mod auth;
mod book;

use bb8::Pool;
use bb8_postgres::{PostgresConnectionManager, bb8};
use tokio_postgres::NoTls;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub pg_pool: Pool<PostgresConnectionManager<NoTls>>,
}

async fn create_connection() -> AppState {
    // set up connection pool
    let pg_manager = PostgresConnectionManager::new_from_stringlike("host=localhost user=cloudcivil dbname=cloudcivil password=miSSion1000", NoTls)
    .unwrap();
    let pg_pool = Pool::builder().build(pg_manager).await.unwrap();

    return AppState{
        pg_pool,
        // redis_pool
    }
}


#[tokio::main]
async fn main() {
    let host = std::env::var("HOST").unwrap();
    let port= std::env::var("PORT").unwrap();
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

    let router  = route::create_router(connection, cors).await;

    // run it
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}"))
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router).await.unwrap();
}

