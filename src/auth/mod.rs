use std::str::FromStr;

use crate::error::AppError;
use crate::AppState;
use axum::{
    extract::State,
    http::{self, header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration as ChronoDuration, Local};
use cookie::{Cookie, CookieBuilder};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::Duration;

#[derive(Deserialize, Debug)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[derive(Deserialize, Debug)]
pub struct SignupForm {
    username: String,
    password: String,
    fname: String,
    lname: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserData {
    user_id: i32,
    fname: String,
    lname: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    aud: Option<String>, // Optional. Audience
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: Option<usize>, // Optional. Issued at (as UTC timestamp)
    iss: Option<String>, // Optional. Issuer
    nbf: Option<usize>, // Optional. Not Before (as UTC timestamp)
    sub: Option<String>, // Optional. Subject (whom token refers to)
    data: UserData,
}

pub async fn login(
    State(pool): State<AppState>,
    Json(body): Json<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    let secret_key = std::env::var("SECRET_KEY").unwrap();

    let conn = pool.pg_pool.get().await?;
    let row = conn
        .query_opt(
            "select user_id, fname, lname, password from users where username=$1",
            &[&body.username],
        )
        .await?;

    if row.is_none() {
        return Err(AppError::InternalServerError("User not found".to_string()));
    }

    let row = row.unwrap();

    let user_id: i32 = row.get(0);
    let fname: String = row.get(1);
    let lname: String = row.get(2);
    let password: String = row.get(3);

    let is_valid_password = verify(&body.password, &password)?;

    if !is_valid_password {
        return Err(AppError::InternalServerError(
            "Invalid password".to_string(),
        ));
    }

    let user_response = json!({
        "user_id": user_id,
        "username": &body.username.clone(),
        "fname": fname.clone(),
        "lname": lname.clone(),
    });
    let current_time = Local::now();
    let expiration_time = current_time + ChronoDuration::days(3);

    let claims = Claims {
        data: UserData {
            fname: fname.clone(),
            lname: lname.clone(),
            user_id: user_id,
        },
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

    let cookie = CookieBuilder::new("Authorization", token)
        .same_site(cookie::SameSite::None)
        .secure(true)
        .http_only(true)
        .max_age(Duration::days(3))
        .path("/")
        .build()
        .to_string();

    let mut header_map = HeaderMap::new();
    header_map.insert(header::SET_COOKIE, cookie.parse().unwrap());

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        header_map,
        Json(user_response),
    ))
}

pub async fn signup(
    State(pool): State<AppState>,
    Json(body): Json<SignupForm>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;

    let row = conn
        .query_opt(
            "select username from users where username=$1",
            &[&body.username],
        )
        .await?;

    if row.is_some() {
        return Err(AppError::InternalServerError("User exists".to_string()));
    }

    let hashed_password = hash(&body.password, DEFAULT_COST)?;
    let _ = conn
        .query(
            "INSERT INTO users(username, password, fname, lname) values($1, $2, $3, $4)",
            &[&body.username, &hashed_password, &body.fname, &body.lname],
        )
        .await?;

    let user_response = json!({
        "username": &body.username.clone(),
        "fname": &body.fname.clone(),
        "lname": &body.lname.clone(),
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        // header_map,
        Json(user_response),
    ))
}

async fn extract_authorization(header: &http::HeaderMap) -> Option<String> {
    if let Some(app_cookies) = header.get("cookie") {
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

pub async fn get_user_session(header: http::HeaderMap) -> Result<impl IntoResponse, AppError> {
    let secret_key = std::env::var("SECRET_KEY").unwrap();

    if let Some(token) = extract_authorization(&header).await {
        let token = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret_key.as_ref()),
            &Validation::default(),
        )?;
        let claims = token.claims;
        return Ok((
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            // header_map,
            Json(claims.data),
        ));
    } else {
        Err(AppError::InternalServerError("Invalid token".to_string()))
    }
}

pub async fn logout() -> Result<impl IntoResponse, AppError> {
    let cookie = CookieBuilder::new("Authorization", "".to_string())
        .same_site(cookie::SameSite::None)
        .secure(true)
        .http_only(true)
        .max_age(Duration::seconds(0))
        .path("/")
        .build()
        .to_string();

    let mut header_map = HeaderMap::new();
    header_map.insert(header::SET_COOKIE, cookie.parse().unwrap());

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        header_map,
        Json(json!({
            "status": "LOGGED_OUT"
        })),
    ))
}
