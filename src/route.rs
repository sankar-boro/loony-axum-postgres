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