use axum::{
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::error::AppError;

pub fn unauthorized(header_map: HeaderMap) -> Result<impl IntoResponse, AppError> {
    Ok((
        StatusCode::UNAUTHORIZED,
        [(header::CONTENT_TYPE, "text/plain")],
        header_map,
        "UnAuthorized",
    ))
}


pub fn no_refresh_token() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "status": "LOGGED_OUT"
        })),
    )
}