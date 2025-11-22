use crate::blog::get::get_home_blogs;
use crate::book::get::{get_book_chapters_and_sections, get_chapter_details, get_home_books, get_section_details};
use crate::connections::session::AppSession;
use crate::user::{get_subscribed_users, subscribe_user, un_subscribe_user};
use crate::{
    blog::{
        delete::{
            delete_blog, delete_blog_node
        },
        append_blog_node, create_blog, edit_blog, edit_blog_node,
        get::{
            get_all_blog_nodes, get_all_blogs_by_page_no, get_all_blogs_by_user_id, get_users_blog,
        },
    },
};
use crate::middleware::require_auth;
use axum::middleware;
use axum::{
    extract::DefaultBodyLimit,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router
};
use tower::ServiceBuilder;
use tower_http::limit::RequestBodyLimitLayer;

use crate::book::{
    create::{append_book_node, create_book}, 
    delete::{delete_book, delete_book_node},
    edit::{edit_book, edit_book_node},
    get::{
        get_all_books_by_page_no, get_all_books_by_user_id,
        get_users_book
    },
};
use crate::file::{get_blog_file, get_book_file, get_tmp_file, upload_file};

use crate::AppState;
use serde_json::json;
use time::Duration;
use crate::connections::cors::init_cors;

pub async fn home() -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({"sankar": "boro"})),
    ))
}

fn blog_routes() -> Router<AppState> {
    Router::new()
        .route("/create", post(create_blog))
        .route("/edit/main", post(edit_blog))
        .route("/edit/node", post(edit_blog_node))
        .route("/delete", post(delete_blog))
        .route("/delete/node", post(delete_blog_node))
        .route("/append/node", post(append_blog_node))
        .route("/get/nodes", get(get_all_blog_nodes))
        .route("/get/:page_no/by_page", get(get_all_blogs_by_page_no))
        .route("/get/:user_id/get_users_blog", get(get_users_blog))
        .route("/get/:uid/user_blogs", get(get_all_blogs_by_user_id))
        .route("/get/home_blogs", get(get_home_blogs))
}

fn book_routes() -> Router<AppState> {
    Router::new()
        .route("/create", post(create_book))
        .route("/edit/main", post(edit_book))
        .route("/edit/node", post(edit_book_node))
        .route("/delete", post(delete_book))
        .route("/delete/node", post(delete_book_node))
        .route("/append/node", post(append_book_node))
        .route("/get/chapter", get(get_chapter_details))
        .route("/get/section", get(get_section_details))
        .route("/get/nav", get(get_book_chapters_and_sections))
        .route("/get/:user_id/get_users_book", get(get_users_book))
        .route("/get/:page_no/by_page", get(get_all_books_by_page_no))
        .route("/get/:uid/user_books", get(get_all_books_by_user_id))
        .route("/get/home_books", get(get_home_books))
}

fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/:user_id/subscribe", post(subscribe_user))
        .route("/:user_id/un_subscribe", post(un_subscribe_user))
        .route("/get_subscribed_users", get(get_subscribed_users))
}

fn file_routes() -> Router<AppState> {
    Router::new()
        .route("/upload", post(upload_file))  
        .route("/blog/:uid/:size/:filename", get(get_blog_file))
        .route("/book/:uid/:size/:filename", get(get_book_file))
        .route("/tmp/:uid/:size/:filename", get(get_tmp_file))
}

pub async fn create_router(app_state: AppState) -> Router {
    let session = AppSession::new(&app_state.config.redis,Duration::days(1)).await;
    let cors = init_cors(&app_state.config.app.allowed_origins);

    Router::new()
        .nest("/blog", blog_routes())
        .nest("/book", book_routes())
        .nest("/user", user_routes())
        .nest("/file", file_routes())
        .route("/", get(home))
        .with_state(app_state.clone())
        .layer(middleware::from_fn_with_state(app_state.clone(), require_auth))
        .layer(cors.clone())
        .layer(session)
        .layer(DefaultBodyLimit::disable())
        .layer(
            ServiceBuilder::new()
                .layer(RequestBodyLimitLayer::new(12 * 1024 * 1024))
                .into_inner(),
        )
}
