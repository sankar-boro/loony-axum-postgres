use axum::http;
use std::str::FromStr;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use cookie::Cookie;

use crate::{error::AppError, types::{Claims}};

pub(crate) fn decode_token(token: &str, auth_app_name: &str, secret_key: &str) -> Result<Claims, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.set_issuer(&[auth_app_name]);
    validation.set_audience(&[auth_app_name]);

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &validation
    )?;

    Ok(token_data.claims)
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
