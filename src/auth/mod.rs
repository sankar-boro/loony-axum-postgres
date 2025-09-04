pub mod utils;

use std::str::FromStr;
use crate::auth::utils::{decode_token, send_access_token, send_refresh_token};
use crate::{auth::utils::generate_token, error::AppError};
use crate::AppState;
use axum::response::Response;
use axum::{
    extract::State,
    http::{self, header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use cookie::{Cookie, CookieBuilder};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::Duration;
use tower_sessions::Session;
use validator::{Validate, ValidationError};

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    static ref PHONE_REGEX: Regex = Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
}

#[derive(Deserialize, Debug, Validate)]
pub struct LoginForm {
    #[validate(custom = "validate_email")]
    email: String,
    #[validate(length(min = 6))]
    password: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct SignupForm {
    #[validate(custom = "validate_email")]
    email: String,
    #[validate(length(min = 6))]
    password: String,
    #[validate(length(min = 3))]
    fname: String,
    lname: String,
}

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

fn validate_email(email: &str) -> Result<(), ValidationError> {
    if EMAIL_REGEX.is_match(email) {
        Ok(())
    } else if PHONE_REGEX.is_match(email) {
        Ok(())
    } else {
        return Err(ValidationError::new("Invalid email."));
    }
}

pub async fn login(
    session: Session,
    State(pool): State<AppState>,
    Json(body): Json<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()?;

    let conn = pool.pg_pool.get().await?;
    let row = conn
        .query_opt(
            "select uid, fname, lname, password from users where email=$1",
            &[&body.email],
        )
        .await?;

    if row.is_none() {
        return Err(AppError::BadRequest(
            serde_json::to_string(
                &json!({ "status": 500, "message": "User not found"})
            ).unwrap()
        ));
    }

    let row = row.unwrap();

    let uid: i32 = row.get(0);
    let fname: String = row.get(1);
    let lname: String = row.get(2);
    let password: String = row.get(3);

    let is_valid_password = verify(&body.password, &password)?;

    if !is_valid_password {
        return Err(AppError::BadRequest(
            serde_json::to_string(
                &json!({ "status": 500, "message": "Invalid password"})
            ).unwrap()
        ));
    }

    let user_response = json!({
        "uid": uid,
        "email": &body.email.clone(),
        "fname": fname.clone(),
        "lname": lname.clone(),
    });

    let mut header_map = HeaderMap::new();
    let token = generate_token(UserData { uid: uid.clone(), fname: fname.clone(), lname: lname.clone() })?;
    send_access_token(&mut header_map, token.clone());
    send_refresh_token(&mut header_map, token);
    session.insert("user_id", uid).await?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        header_map,
        Json(user_response),
    ))
}

pub async fn signup(
    State(state): State<AppState>,
    Json(body): Json<SignupForm>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()?;
    let conn = state.pg_pool.get().await?;

    let row = conn
        .query_opt(
            "select email from users where email=$1",
            &[&body.email],
        )
        .await?;

    if row.is_some() {
        return Err(AppError::BadRequest(
            serde_json::to_string(
                &json!({ "status": 500, "message": "User exists"})
            ).unwrap()
        ));
    }

    let hashed_password = hash(&body.password, DEFAULT_COST)?;
    conn
        .execute(
            "INSERT INTO users(email, password, fname, lname) values($1, $2, $3, $4) RETURNING uid",
            &[&body.email, &hashed_password, &body.fname, &body.lname],
        )
        .await?;

    let user_response = json!({
        "email": &body.email.clone(),
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

pub async fn refresh_token(session: Session, header: HeaderMap) -> Result<Response, AppError> {
    // Example: check for Authorization header
    let cookies = parse_cookies(&header);

    let mut refresh_token = String::from("");
    for (name, value) in cookies.iter() {
            if name == "refresh_token" {
                refresh_token = value.to_string();
            }
        }
    let refresh_token = decode_token(&refresh_token)?;
    let token = generate_token(refresh_token.data.clone())?;
    let mut header_map = HeaderMap::new();
    send_access_token(&mut header_map, token.clone());
    send_refresh_token(&mut header_map, token);
    session.insert("user_id", refresh_token.data.uid).await.unwrap();
     Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        header_map,
        Json(json!({
            "status": "Ok"
        }))
    ).into_response())
}


pub async fn logout() -> Result<impl IntoResponse, AppError> {
    let access_token = CookieBuilder::new("access_token", "".to_string())
        .same_site(cookie::SameSite::None)
        .secure(true)
        .http_only(true)
        .max_age(Duration::seconds(0))
        .path("/")
        .build()
        .to_string();

    let refresh_token = CookieBuilder::new("refresh_token", "".to_string())
        .same_site(cookie::SameSite::None)
        .secure(true)
        .http_only(true)
        .max_age(Duration::seconds(0))
        .path("/")
        .build()
        .to_string();

    let mut header_map = HeaderMap::new();
    header_map.insert(header::SET_COOKIE, access_token.parse().unwrap());
    header_map.insert(header::SET_COOKIE, refresh_token.parse().unwrap());

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        header_map,
        Json(json!({
            "status": "LOGGED_OUT"
        })),
    ))
}

#[derive(Deserialize, Debug, Validate)]
pub struct ResetPassword {
    session_id: String,
    #[validate(custom = "validate_email")]
    email: String,
    #[validate(length(min = 6))]
    password: String,
    #[validate(length(min = 6))]
    confirm_password: String
}

pub async fn reset_password(
    session: Session,
    State(state): State<AppState>,
    Json(body): Json<ResetPassword>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()?;

    let session_id = session
        .get::<String>("RESET_PASSWORD_SESSION_ID")
        .await?;

    if session_id.is_none() {
        return Err(AppError::BadRequest(
            serde_json::to_string(
                &json!({ "status": 500, "message": "Email reset link has been expired."})
            ).unwrap()
        ));
    }

    if let Some(session_id) = session_id {
        if session_id != body.session_id {
            return Err(AppError::BadRequest(
                serde_json::to_string(
                    &json!({ "status": 500, "message": "Email reset link has been expired."})
                ).unwrap()
            ));
        }
    }

    let conn = state.pg_pool.get().await?;

    let row = conn
        .query_opt(
            "select email from users where email=$1",
            &[&body.email],
        )
        .await?;

    if row.is_none() {
        return Err(AppError::BadRequest(
            serde_json::to_string(
                &json!({ "status": 500, "message": "User does not exist."})
            ).unwrap()
        ));
    }

    let hashed_password = hash(&body.password, DEFAULT_COST)?;
    conn
        .execute(
            "UPDATE users SET password=$1 where email=$2",
            &[&hashed_password, &body.email],
        )
        .await?;

    let user_response = json!({
        "status": 200,
        "message": "Password has been reset successfully",
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        // header_map,
        Json(user_response),
    ))
}



pub async fn user_info(
    session: Session,
    State(state): State<AppState>
) -> Result<Response, AppError> {
    
    let user: Option<i32> = session.get("user_id").await?;
    let user_id = match user {
        Some(user) => user,
        None => {
            return Err(AppError::NotFound("User not found".to_string()));
        }
    };

    let conn = state.pg_pool.get().await?;

    let row = conn
        .query_one(
            "select uid, fname, lname, email from users where uid=$1",
            &[&user_id],
        )
        .await?;

    let uid: i32 = row.get(0);
    let fname: String = row.get(1);
    let lname: String = row.get(2);
    let email: String = row.get(3);

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "uid": uid,
            "fname": fname,
            "lname": lname,
            "email": email
        })),
    ).into_response())
}