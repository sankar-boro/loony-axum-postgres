use axum::{Router, routing::{get, 
    post
}};

use tower_http::cors::CorsLayer;
use time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};
use crate::AppState;

use crate::auth::{login, signup};
use crate::book::create_book;


pub async fn create_router(connection: AppState, cors: CorsLayer) -> Router {
    let auth = Router::new().route("/login", get(login).post(login))
    .route("/signup", post(signup))
    .route("/create_book", post(create_book));

    let pool = RedisPool::new(RedisConfig::from_url("redis://:sankar@127.0.0.1:6379/").unwrap(), None, None, None, 6).unwrap();

    pool.connect();
    pool.wait_for_connect().await.unwrap();

    let session_store = RedisStore::new(pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(30)));

     Router::new()
        .nest("/api/auth", auth)
        .route(
            "/login",
            post(login),
        )
        .with_state(connection).layer(cors).layer(session_layer)
}