
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
mod config;
mod init;
mod auth;
mod middleware;
mod connections;
mod tracing_setup;

pub(crate) use init::AppState;
use std::net::SocketAddr;
use axum_server::tls_rustls::RustlsConfig;

#[tokio::main]
async fn main() {

    tracing_setup::init_tracing();
    tracing::info!("Application started!");

    let app_state = init::init_app_state().await;

    let http_port = app_state.config.app.http_port.to_owned();
    let https_port = app_state.config.app.https_port.to_owned();

    let router = route::create_router(app_state.clone()).await;
    
    let http = tokio::spawn(http_server(router.clone(), http_port));
    let https = tokio::spawn(https_server(router, https_port));

    let _ = tokio::join!(http, https);
}


async fn http_server(router: axum::Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Http server started at port {}", addr.port());
    axum_server::bind(addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn https_server(router: axum::Router, port: u16) {
      let config = RustlsConfig::from_pem_file(
        ".local/localhost.pem",
        ".local/localhost-key.pem",
    )
    .await
    .unwrap();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Https server started at {}", addr.port());
    axum_server::bind_rustls(addr, config)
        .serve(router.into_make_service())
        .await
        .unwrap();
}