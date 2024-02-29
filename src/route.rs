use axum::{Router, routing::{get, 
    post
}};
use axum::{
    // extract::State,
    // http::{header, Response, StatusCode},
    http::StatusCode,
    response::IntoResponse,
    // Extension, 
    Json,
};
use tower_http::cors::CorsLayer;

use crate::AppState;

use crate::auth::login;

pub fn create_router(connection: AppState, cors: CorsLayer) -> Router {
    let auth = Router::new().route("/login", get(login).post(login));

     Router::new()
        .nest("/api/auth", auth)
        .route(
            "/login",
            get(login),
        )
        .with_state(connection).layer(cors)
}