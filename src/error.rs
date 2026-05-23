use std::{env::VarError, num::ParseIntError};

use axum::{
    extract::multipart::MultipartError,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bcrypt::BcryptError;
use image::ImageError;
use validator::ValidationErrors;

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    InternalServerError(String),
    Error((StatusCode, String)),
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

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}

impl From<BcryptError> for AppError {
    fn from(err: BcryptError) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}
impl From<MultipartError> for AppError {
    fn from(err: MultipartError) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}
impl From<ImageError> for AppError {
    fn from(err: ImageError) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}
impl From<tower_sessions::session::Error> for AppError {
    fn from(err: tower_sessions::session::Error) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}
impl From<VarError> for AppError {
    fn from(err: VarError) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}
impl From<ValidationErrors> for AppError {
    fn from(err: ValidationErrors) -> Self {
        AppError::InternalServerError(
            serde_json::to_string(&err.field_errors())
                .unwrap_or("Failed to serialize ValidationErrors".to_string()),
        )
    }
}
impl From<ParseIntError> for AppError {
    fn from(err: ParseIntError) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::InternalServerError(err.to_string())
    }
}
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(e) => write!(f, "not found: {}", e),
            AppError::BadRequest(e) => write!(f, "bad request: {}", e),
            AppError::InternalServerError(e) => write!(f, "internal server error: {}", e),
            AppError::Error((status, msg)) => write!(f, "error {}: {}", status, msg),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            AppError::BadRequest(e) => (StatusCode::BAD_REQUEST, e).into_response(),
            AppError::InternalServerError(e) => {
                tracing::error!(error = %e, "internal server error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            }
            AppError::Error((status_code, msg)) => (status_code, msg).into_response(),
        }
    }
}
