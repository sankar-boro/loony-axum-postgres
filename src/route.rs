use crate::blog::get::get_home_blogs;
use crate::book::get::{get_book_chapters_and_sections, get_chapter_details, get_home_books, get_section_details};
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
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router
};
use tower::ServiceBuilder;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::book::{
    create::{append_book_node, create_book},
    delete::{delete_book, delete_book_node},
    edit::{edit_book, edit_book_node},
    get::{get_all_books_by_page_no, get_all_books_by_user_id, get_users_book},
    upload::upload_book,
};
use crate::file::{get_blog_file, get_book_file, get_tmp_file, upload_file};

use crate::AppState;
use serde_json::json;
use crate::connections::cors::init_cors;

pub async fn home() -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({"sankar": "boro"})),
    ))
}

// Public read routes — no authentication required
fn blog_read_routes() -> Router<AppState> {
    Router::new()
        .route("/get/nodes", get(get_all_blog_nodes))
        .route("/get/:page_no/by_page", get(get_all_blogs_by_page_no))
        .route("/get/:user_id/get_users_blog", get(get_users_blog))
        .route("/get/:uid/user_blogs", get(get_all_blogs_by_user_id))
        .route("/get/home_blogs", get(get_home_blogs))
}

fn book_read_routes() -> Router<AppState> {
    Router::new()
        .route("/get/chapter", get(get_chapter_details))
        .route("/get/section", get(get_section_details))
        .route("/get/nav", get(get_book_chapters_and_sections))
        .route("/get/:user_id/get_users_book", get(get_users_book))
        .route("/get/:page_no/by_page", get(get_all_books_by_page_no))
        .route("/get/:uid/user_books", get(get_all_books_by_user_id))
        .route("/get/home_books", get(get_home_books))
}

fn file_read_routes() -> Router<AppState> {
    Router::new()
        .route("/blog/:uid/:size/:filename", get(get_blog_file))
        .route("/book/:uid/:size/:filename", get(get_book_file))
        .route("/tmp/:uid/:size/:filename", get(get_tmp_file))
}

// Authenticated write routes — require a valid session
fn blog_write_routes() -> Router<AppState> {
    Router::new()
        .route("/create", post(create_blog))
        .route("/edit/main", post(edit_blog))
        .route("/edit/node", post(edit_blog_node))
        .route("/delete", post(delete_blog))
        .route("/delete/node", post(delete_blog_node))
        .route("/append/node", post(append_blog_node))
}

fn book_write_routes() -> Router<AppState> {
    Router::new()
        .route("/create", post(create_book))
        .route("/edit/main", post(edit_book))
        .route("/edit/node", post(edit_book_node))
        .route("/delete", post(delete_book))
        .route("/delete/node", post(delete_book_node))
        .route("/append/node", post(append_book_node))
        // ZIP upload — override body limit to 50 MB for this route only
        .route(
            "/upload",
            post(upload_book).layer(DefaultBodyLimit::max(50 * 1024 * 1024)),
        )
}

fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/:user_id/subscribe", post(subscribe_user))
        .route("/:user_id/un_subscribe", post(un_subscribe_user))
        .route("/get_subscribed_users", get(get_subscribed_users))
}

fn file_write_routes() -> Router<AppState> {
    Router::new()
        .route("/upload", post(upload_file))
}

pub async fn create_router(app_state: AppState) -> Router {
    let cors = init_cors(&app_state.config.app.allowed_origins);

    let security_headers = ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=63072000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ));

    // Routes that require a valid access_token cookie
    let protected = Router::new()
        .nest("/blog", blog_write_routes())
        .nest("/book", book_write_routes())
        .nest("/user", user_routes())
        .nest("/file", file_write_routes())
        .with_state(app_state.clone())
        .layer(middleware::from_fn_with_state(app_state.clone(), require_auth));

    // Routes accessible without authentication
    let public = Router::new()
        .nest("/blog", blog_read_routes())
        .nest("/book", book_read_routes())
        .nest("/file", file_read_routes())
        .route("/", get(home))
        .with_state(app_state.clone());

    Router::new()
        .merge(public)
        .merge(protected)
        .layer(cors)
        .layer(DefaultBodyLimit::disable())
        .layer(
            ServiceBuilder::new()
                .layer(RequestBodyLimitLayer::new(12 * 1024 * 1024))
                .into_inner(),
        )
        .layer(security_headers)
}
