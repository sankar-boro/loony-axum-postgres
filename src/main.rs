
mod blog;
mod book;
mod error;
mod file;
mod route;
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

#[tokio::main]
async fn main() {

    tracing_setup::init_tracing();
    
    let app_state = init::init_app_state().await;

    let port = app_state.port().to_owned();
    let app_env = app_state.app_env().to_owned();

    let router = route::create_router(app_state.clone()).await;
    
    let host = if app_env == "production" {
        "0.0.0.0" 
    } else {
        "localhost"
    };
    
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await.unwrap();
    tracing::info!("Application started at {}:{}", host, port);
    axum::serve(listener, router).await.unwrap();
}