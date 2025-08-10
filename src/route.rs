use crate::blog::get::get_home_blogs;
use crate::book::get::{get_book_chapters_and_sections, get_chapter_details, get_home_books, get_section_details};
use crate::{auth, mail};
use crate::user::{get_subscribed_users, subscribe_user, un_subscribe_user};
use crate::{
    auth::logout,
    blog::{
        delete::{
            delete_blog, delete_blog_node
        },
        append_blog_node, create_blog, edit_blog, edit_blog_node,
        get::{
            get_all_blog_nodes, get_all_blogs_by_page_no, get_all_blogs_by_user_id, get_users_blog,
        },
    },
    likes::tag::{
        get_all_tags_user_can_follow, get_all_tags_user_has_followed, user_followed_a_tag,
        user_removed_a_followed_tag,
    },
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
    create::{append_book_node, create_book}, 
    delete::{delete_book, delete_book_node},
    edit::{edit_book, edit_book_node},
    get::{
        get_all_books_by_page_no, get_all_books_by_user_id,
        get_users_book
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

    let session_store = RedisStore::new(pool.clone());
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(3)));

    let login_routes = Router::new()
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
        .route("/get/:user_id/get_users_blog", get(get_users_blog))
        .route("/get/:uid/user_blogs", get(get_all_blogs_by_user_id))
        .route("/get/home_blogs", get(get_home_blogs));

    let book_routes = Router::new()
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
        .route("/get/home_books", get(get_home_books));

    let user_routes = Router::new()
        .route("/:user_id/subscribe", post(subscribe_user))
        .route("/:user_id/un_subscribe", post(un_subscribe_user))
        .route("/get_subscribed_users", get(get_subscribed_users));

    let tag_routes = Router::new()
        .route(
            "/:user_id/get_all_tags_user_can_follow",
            get(get_all_tags_user_can_follow),
        )
        .route(
            "/:user_id/get_all_tags_user_has_followed",
            get(get_all_tags_user_has_followed),
        )
        .route("/user_followed_a_tag", post(user_followed_a_tag))
        .route(
            "/user_removed_a_followed_tag",
            post(user_removed_a_followed_tag),
        );

    let auth_routes = Router::new()
        .nest("/v1/auth", login_routes)
        .nest("/v1/blog", blog_routes)
        .nest("/v1/book", book_routes)
        .nest("/v1/tag", tag_routes)
        .nest("/v1/user", user_routes)
        .route("/v1/upload_file", post(upload_file))
        .route("/v1/blog/:uid/:size/:filename", get(get_blog_file))
        .route("/v1/book/:uid/:size/:filename", get(get_book_file))
        .route("/v1/tmp/:uid/:size/:filename", get(get_tmp_file))
        .route("/v1", get(home))
        .with_state(connection.clone())
        .layer(cors.clone())
        .layer(session_layer.clone())
        .layer(DefaultBodyLimit::disable())
        .layer(
            ServiceBuilder::new()
                .layer(RequestBodyLimitLayer::new(12 * 1024 * 1024))
                .into_inner(),
        );

    let session_store = RedisStore::new(pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::minutes(10)));
    let un_auth_routes = Router::new()
        .route("/v1/mail", post(mail::send_email))
        .route("/v1/reset_password", post(auth::reset_password))
        .with_state(connection)
        .layer(cors)
        .layer(session_layer)
        .layer(DefaultBodyLimit::disable())
        .layer(
            ServiceBuilder::new()
                .layer(RequestBodyLimitLayer::new(12 * 1024 * 1024))
                .into_inner(),
        );

    un_auth_routes.merge(auth_routes)
}
