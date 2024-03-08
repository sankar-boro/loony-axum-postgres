use axum::{http::{StatusCode, header}, response::IntoResponse, Json, routing::{get, 
    post
}, Router};

use serde_json::json;
use tower_http::cors::CorsLayer;
use time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};
use crate::{auth::{get_user_session, login, signup}, AppState};
use crate::book::create_book;

pub async fn home() -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        // header_map,
        Json(json!({"sankar": "boro"})),
    ))
}

pub async fn create_router(connection: AppState, cors: CorsLayer) -> Router {

    let pool = RedisPool::new(RedisConfig::from_url("redis://:sankar@127.0.0.1:6379/").unwrap(), None, None, None, 6).unwrap();

    pool.connect();
    pool.wait_for_connect().await.unwrap();

    let session_store = RedisStore::new(pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(30)));

    let auth_routes = Router::new()
        .route("/login", get(login).post(login))
        .route("/signup", post(signup))
        .route("/user/session", get(get_user_session).post(get_user_session));

    let book_routes = Router::new()
        .route("/create_book", post(create_book));

    Router::new()
        .nest("/api/auth", auth_routes)
        .nest("/api/book", book_routes)
        .route(
            "/",
            get(home),
        )
        .with_state(connection).layer(cors).layer(session_layer)
}