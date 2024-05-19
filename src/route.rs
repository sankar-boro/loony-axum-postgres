use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use crate::{
    auth::logout,
    blog::{
        append_blog_node, create_blog, delete_blog, delete_blog_node, edit_blog, edit_blog_node,
        get_all_blog_nodes, get_all_blogs,
    },
    book::test_query,
};

use crate::book::{
    append_book_node, create_book, delete_book, delete_book_node, edit_book, edit_book_node,
    get_all_books, get_book_chapters, get_book_sections, get_book_sub_sections,
};
use crate::file::{get_file, get_uploaded_file, upload_file};

use crate::{
    auth::{get_user_session, login, signup},
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
        )
        .route("/logout", post(logout));

    let blog_routes = Router::new()
        .route("/create", post(create_blog))
        .route("/edit_blog", post(edit_blog))
        .route("/edit_blog_node", post(edit_blog_node))
        .route("/delete_blog", post(delete_blog))
        .route("/delete_blog_node", post(delete_blog_node))
        .route("/append_blog_node", post(append_blog_node))
        .route("/get_all_blogs", get(get_all_blogs))
        .route("/get_all_blog_nodes", get(get_all_blog_nodes));

    let book_routes = Router::new()
        .route("/create", post(create_book))
        .route("/edit_book", post(edit_book))
        .route("/edit_book_node", post(edit_book_node))
        .route("/delete_book", post(delete_book))
        .route("/delete_book_node", post(delete_book_node))
        .route("/append_book_node", post(append_book_node))
        .route("/get_all_books", get(get_all_books))
        .route("/get_all_book_nodes", get(get_book_chapters))
        .route("/get_book_chapters", get(get_book_chapters))
        .route("/get_book_sections", get(get_book_sections))
        .route("/get_book_sub_sections", get(get_book_sub_sections));

    Router::new()
        .nest("/api/auth", auth_routes)
        .nest("/api/blog", blog_routes)
        .nest("/api/book", book_routes)
        .route("/api/upload_file", post(upload_file))
        .route("/api/i/:filename", get(get_file))
        .route("/api/u/:filename", get(get_uploaded_file))
        .route("/", get(home))
        .route("/test_query", get(test_query))
        .with_state(connection)
        .layer(cors)
        .layer(session_layer)
}
