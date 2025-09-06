use crate::error::AppError;
use axum::http::HeaderMap;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserData {
    pub uid: i32,
    pub fname: String,
    pub lname: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    aud: Option<String>, // Optional. Audience
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: Option<usize>, // Optional. Issued at (as UTC timestamp)
    iss: Option<String>, // Optional. Issuer
    nbf: Option<usize>, // Optional. Not Before (as UTC timestamp)
    sub: Option<String>, // Optional. Subject (whom token refers to)
    pub data: UserData,
}
use axum::http::{ Request};
use axum::response::{IntoResponse, Response};
use axum::{
    middleware::Next,
};
use reqwest::StatusCode;

fn parse_cookies(headers: &HeaderMap) -> Vec<(String, String)> {
    let mut cookies = Vec::new();

    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for pair in cookie_str.split(';') {
                let mut parts = pair.trim().splitn(2, '=');
                let name = parts.next().unwrap_or("").trim();
                let value = parts.next().unwrap_or("").trim();
                if !name.is_empty() {
                    cookies.push((name.to_string(), value.to_string()));
                }
            }
        }
    }

    cookies
}

pub async fn require_auth(header: HeaderMap, req: Request<axum::body::Body>, next: Next) -> Result<Response, AppError> {
    let cookies = parse_cookies(&header);
    for (name, value) in cookies.iter() {
        if name == "access_token" {
            match decode_token(value) {
                Ok(_) => {
                    return Ok(next.run(req).await)
                },
                Err(_) => {
                    return Ok(AppError::Error((StatusCode::UNAUTHORIZED, "UnAuthorized".to_string())).into_response());
                },
            }
        }
    }
    return Ok(AppError::Error((StatusCode::UNAUTHORIZED, "UnAuthorized".to_string())).into_response());
}

pub(crate) fn decode_token(token: &str) -> Result<Claims, AppError> {
    let secret_key = std::env::var("SECRET_KEY").unwrap();

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::default(), // you can customize validation if needed
    )?;

    Ok(token_data.claims)
}
