use crate::error::AppError;
use axum::http::HeaderMap;
use chrono::{Duration as ChronoDuration, Local};
use jsonwebtoken::{encode, decode, DecodingKey, Validation, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use cookie::CookieBuilder;
use time::Duration;
use axum::http::header;
use super::UserData;

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

pub(crate) fn generate_token(data: UserData) -> Result<String, AppError>{
    let secret_key = std::env::var("SECRET_KEY").unwrap();
    let current_time = Local::now();
    let expiration_time = current_time + ChronoDuration::days(3);
        let claims = Claims {
        data,
        exp: expiration_time.timestamp() as usize,
        aud: None,
        iat: Some(current_time.timestamp() as usize),
        iss: None,
        nbf: None,
        sub: None,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )?;

    Ok(token)
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

pub(crate) fn send_access_token(header: &mut HeaderMap, token: String) {
    let access_token = CookieBuilder::new("access_token", token)
        .same_site(cookie::SameSite::Lax)
        .secure(false) // On https, value should be true
        .http_only(true)
        .max_age(Duration::days(1))
        .path("/")
        .build()
        .to_string();
    header.append(header::SET_COOKIE, access_token.parse().unwrap());
}

pub(crate) fn send_refresh_token(header: &mut HeaderMap, token: String) {
    let refresh_token = CookieBuilder::new("refresh_token", token)
        .same_site(cookie::SameSite::Lax)
        .secure(false) // On https, value should be true
        .http_only(true)
        .max_age(Duration::minutes(30))
        .path("/")
        .build()
        .to_string();
    header.append(header::SET_COOKIE, refresh_token.parse().unwrap());
}
