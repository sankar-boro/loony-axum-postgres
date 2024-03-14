use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use crate::book::{append_book_node, create_book, delete_book, edit_book, get_all_book_nodes};
use crate::{
    auth::{get_user_session, login, signup},
    book::get_all_books,
    AppState,
};
use serde_json::json;
use time::Duration;
use tower_http::cors::CorsLayer;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};

pub async fn home() -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        // header_map,
        Json(json!({"sankar": "boro"})),
    ))
}

pub async fn create_router(connection: AppState, cors: CorsLayer) -> Router {
    let pool = RedisPool::new(
        RedisConfig::from_url("redis://:sankar@127.0.0.1:6379/").unwrap(),
        None,
        None,
        None,
        6,
    )
    .unwrap();

    pool.connect();
    pool.wait_for_connect().await.unwrap();

    let session_store = RedisStore::new(pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(30)));

    let auth_routes = Router::new()
        .route("/login", get(login).post(login))
        .route("/signup", post(signup))
        .route(
            "/user/session",
            get(get_user_session).post(get_user_session),
        );

    let book_routes = Router::new()
        .route("/create_book", post(create_book))
        .route("/edit_book", post(edit_book))
        .route("/delete_book", post(delete_book))
        .route("/append_book_node", post(append_book_node))
        .route("/get_all_books", get(get_all_books))
        .route("/get_all_book_nodes", get(get_all_book_nodes));

    Router::new()
        .nest("/api/auth", auth_routes)
        .nest("/api/book", book_routes)
        .route("/", get(home))
        .with_state(connection)
        .layer(cors)
        .layer(session_layer)
}
