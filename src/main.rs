mod auth;
mod blog;
mod book;
mod error;
mod file;
mod likes;
mod route;
mod search;
mod traits;
mod types;
mod user;
mod utils;
mod query;
mod mail;
mod config;
mod init;
mod middleware;

pub(crate) use init::AppState;
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};



#[tokio::main]
async fn main() {

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_tokio_postgres=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app_state = init::init().await;
    let allowed_origins = app_state.app_config.allowed_origins.clone();
    let app_hostname = &app_state.app_config.hostname;
    let app_port = &app_state.app_config.port;


    // Parse the comma-separated string into a Vec<String>
    let origins: Vec<HeaderValue> = allowed_origins
        .split(',')
        .map(|s| s.parse::<HeaderValue>().unwrap())
        .collect();
    log::debug!("Origins: {:?}", origins);
    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE, ACCESS_CONTROL_ALLOW_ORIGIN]);

    let router = route::create_router(app_state.clone(), cors).await;
    let listener = tokio::net::TcpListener::bind(format!("{app_hostname}:{app_port}"))
        .await
        .unwrap();
    tracing::info!("Server is listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, router).await.unwrap();
}
