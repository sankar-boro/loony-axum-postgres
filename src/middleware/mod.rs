use crate::error::AppError;
use axum::http::HeaderMap;
use crate::auth::decode_token;
use axum::extract::State;
use crate::AppState;

use axum::http::{ Request};
use axum::response::{IntoResponse, Response};
use axum::{
    middleware::Next,
};
use reqwest::StatusCode;


pub fn parse_cookies<'a>(headers: &'a HeaderMap) -> Vec<(&'a str, &'a str)> {
    headers
        .get_all("cookie")
        .iter()
        .filter_map(|val| val.to_str().ok())
        .flat_map(|s| s.split(';'))
        .filter_map(|pair| {
            let mut parts = pair.trim().splitn(2, '=');
            let name = parts.next()?.trim();
            if name.is_empty() {
                return None;
            }
            let value = parts.next().unwrap_or("").trim();
            Some((name, value))
        })
        .collect()
}

pub async fn require_auth(State(state): State<AppState>, header: HeaderMap, req: Request<axum::body::Body>, next: Next) -> Result<Response, AppError> {
    let cookies = parse_cookies(&header);
    for (name, value) in cookies.iter() {
        if *name == "access_token" {
            match decode_token(value, &state.config.app.auth_app_name, &state.config.app.secret_key) {
                Ok(_) => {
                    return Ok(next.run(req).await)
                },
                Err(e) => {
                    return Ok(AppError::Error((StatusCode::UNAUTHORIZED, "UnAuthorized".to_string())).into_response());
                },
            }
        }
    }
    return Ok(AppError::Error((StatusCode::UNAUTHORIZED, "UnAuthorized".to_string())).into_response());
}

