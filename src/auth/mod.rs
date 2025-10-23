use axum::http::{self, HeaderMap};
use std::str::FromStr;
use chrono::{Duration as ChronoDuration, Local};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use cookie::{Cookie, CookieBuilder};
use time::Duration;

use crate::{error::AppError, types::{Claims, UserClaims}};

pub(crate) fn generate_token(data: UserClaims, app_name: &str, secret_key: &str) -> Result<String, AppError> {
    let current_time = Local::now();
    let expiration_time = current_time + ChronoDuration::days(3);
        let claims = Claims {
        data,
        exp: expiration_time.timestamp() as usize,
        iat: Some(current_time.timestamp() as usize),
        iss: Some(app_name.to_string()),
        aud: Some(app_name.to_string()),
        nbf: None,
        sub: None,
    };

    let header = Header {
        alg: Algorithm::HS256,
        typ: Some("JWT".to_string()),
        ..Default::default()
    };
    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )?;

    Ok(token)
}

pub(crate) fn decode_token(token: &str, app_name: &str, secret_key: &str) -> Result<Claims, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.set_issuer(&[app_name]);
    validation.set_audience(&[app_name]);

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &validation
    )?;

    Ok(token_data.claims)
}

pub(crate) fn cookie(name: &str, value: &str, duration: Duration) -> String {
    CookieBuilder::new(name, value)
        .same_site(cookie::SameSite::None)
        .secure(true)
        .http_only(true)
        .max_age(duration)
        .path("/")
        .build()
        .to_string()
}

pub fn parse_cookies(headers: &HeaderMap) -> Vec<(String, String)> {
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

#[allow(unused)]
pub async fn extract_authorization(header: &http::HeaderMap) -> Option<String> {
    if let Some(app_cookies) = header.get("session_token") {
        // Parse the cookie string
        let cookies: Vec<Cookie<'_>> = app_cookies
            .to_str()
            .unwrap()
            .split("; ")
            .map(|s| Cookie::from_str(s).unwrap())
            .collect();

        // Iterate over the parsed cookies
        for cookie in cookies {
            if cookie.name() == "Authorization" {
                let token = cookie.value().to_string();
                return Some(token);
            }
        }
    }
    None
}
