use axum::{http::StatusCode, Json};
use serde_json::json;

pub fn internal_error<E>(err: E) -> (StatusCode, Json<serde_json::Value>)
where
    E: std::error::Error,
{
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "message": err.to_string(),
        })),
    )
}
