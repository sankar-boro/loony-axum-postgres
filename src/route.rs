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
use axum::{
    extract::State,
    http::{header, Response},
    Extension,
};
use time::Duration;
use tower_sessions::{Expiry, Session, SessionManagerLayer};
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};
use crate::AppState;

use crate::auth::login;

// pub async fn other_login(
//     State(pool): State<AppState>,
// ) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)>  {
//     let user_response = serde_json::json!({"status": "success","data": serde_json::json!({
//         "user": "Sankar".to_string()
//     })});
//     Ok(Json(user_response))

// }

pub async fn create_router(connection: AppState, cors: CorsLayer) -> Router {
    let auth = Router::new().route("/login", get(login).post(login));
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
            get(login),
        )
        .with_state(connection).layer(cors).layer(session_layer)
}