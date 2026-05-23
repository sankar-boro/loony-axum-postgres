use crate::error::AppError;
use crate::utils::UserId;
use axum::http::HeaderMap;
use crate::auth::decode_token;
use axum::extract::State;
use crate::AppState;

use axum::http::Request;
use axum::response::{IntoResponse, Response};
use axum::middleware::Next;
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

pub async fn require_auth(
    State(state): State<AppState>,
    header: HeaderMap,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    let cookies = parse_cookies(&header);
    for (name, value) in cookies.iter() {
        if *name == "access_token" {
            match decode_token(value, &state.config.app.auth_app_name, &state.config.app.secret_key) {
                Ok(claims) => {
                    let user_id: i32 = claims.sub
                        .parse()
                        .map_err(|_| AppError::Error((StatusCode::UNAUTHORIZED, "Invalid token subject".into())))?;
                    req.extensions_mut().insert(UserId(user_id));
                    return Ok(next.run(req).await);
                }
                Err(_) => {
                    return Ok(AppError::Error((StatusCode::UNAUTHORIZED, "Unauthorized".into())).into_response());
                }
            }
        }
    }
    Ok(AppError::Error((StatusCode::UNAUTHORIZED, "access_token not found".into())).into_response())
}

