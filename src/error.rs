use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub enum AppError {
    // NotFound(String),
    InternalServerError(String),
}

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

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}

impl From<tokio_postgres::Error> for AppError {
    fn from(err: tokio_postgres::Error) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}

impl From<bb8::RunError<tokio_postgres::Error>> for AppError {
    fn from(err: bb8::RunError<tokio_postgres::Error>) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = match self {
            // AppError::NotFound(e) => e,
            AppError::InternalServerError(e) => e,
        };
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
