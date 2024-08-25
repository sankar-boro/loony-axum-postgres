use crate::{
    auth::logout,
    blog::{
        append_blog_node, create_blog, delete_blog, delete_blog_node, edit_blog, edit_blog_node,
        get::{
            get_all_blog_nodes, get_all_blogs_by_page_no, get_all_blogs_by_user_id,
            get_all_blogs_liked_by_user,
        },
    },
    book::{get::get_all_books_liked_by_user, test_query},
    likes::tag::get_all_tags_user_can_follow,
};
use axum::{
    extract::DefaultBodyLimit,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tower::ServiceBuilder;
use tower_http::limit::RequestBodyLimitLayer;

use crate::book::{
    append_book_node, create_book, delete_book, delete_book_node,
    edit::{edit_book, edit_book_node},
    get::{
        get_all_books_by_page_no, get_all_books_by_user_id, get_book_chapters, get_book_sections,
        get_book_sub_sections,
    },
};
use crate::file::{get_blog_file, get_book_file, get_tmp_file, upload_file};

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
        .with_expiry(Expiry::OnInactivity(Duration::days(3)));

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
        .route("/edit/main", post(edit_blog))
        .route("/edit/node", post(edit_blog_node))
        .route("/delete", post(delete_blog))
        .route("/delete/node", post(delete_blog_node))
        .route("/append/node", post(append_blog_node))
        .route("/get/nodes", get(get_all_blog_nodes))
        .route("/get/:page_no/by_page", get(get_all_blogs_by_page_no))
        .route(
            "/get/:user_id/get_all_blogs_liked_by_user",
            get(get_all_blogs_liked_by_user),
        )
        .route("/get/:uid/user_blogs", get(get_all_blogs_by_user_id));

    let book_routes = Router::new()
        .route("/create", post(create_book))
        .route("/edit/main", post(edit_book))
        .route("/edit/node", post(edit_book_node))
        .route("/delete", post(delete_book))
        .route("/delete/node", post(delete_book_node))
        .route("/append/node", post(append_book_node))
        .route("/get/nodes", get(get_book_chapters))
        .route("/get/chapters", get(get_book_chapters))
        .route("/get/sections", get(get_book_sections))
        .route("/get/sub_sections", get(get_book_sub_sections))
        .route(
            "/get/:user_id/get_all_books_liked_by_user",
            get(get_all_books_liked_by_user),
        )
        .route("/get/:page_no/by_page", get(get_all_books_by_page_no))
        .route("/get/:uid/user_books", get(get_all_books_by_user_id));

    let tag_routes = Router::new().route(
        "/:user_id/get_all_tags_user_can_follow",
        get(get_all_tags_user_can_follow),
    );

    Router::new()
        .nest("/api/auth", auth_routes)
        .nest("/api/blog", blog_routes)
        .nest("/api/book", book_routes)
        .nest("/api/tag", tag_routes)
        .route("/api/upload_file", post(upload_file))
        .route("/api/blog/:uid/:size/:filename", get(get_blog_file))
        .route("/api/book/:uid/:size/:filename", get(get_book_file))
        .route("/api/tmp/:uid/:size/:filename", get(get_tmp_file))
        .route("/", get(home))
        .route("/test_query", get(test_query))
        .with_state(connection)
        .layer(cors)
        .layer(session_layer)
        .layer(DefaultBodyLimit::disable())
        .layer(
            ServiceBuilder::new()
                .layer(RequestBodyLimitLayer::new(12 * 1024 * 1024))
                .into_inner(),
        )
}
