use crate::AppState;
use axum::{
    extract::State,
    http::{self, header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use cookie::CookieBuilder;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::Duration;
use tower_sessions::Session;

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
    data: UserData,
    exp: usize,
}

fn internal_error<E>(err: E) -> (StatusCode, Json<serde_json::Value>)
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

pub async fn login(
    State(pool): State<AppState>,
    _: Session,
    Json(body): Json<LoginForm>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let secret_key = std::env::var("SECRET_KEY").unwrap();

    let conn = pool.pg_pool.get().await.map_err(internal_error)?;
    let row = conn
        .query_one(
            "select user_id, fname, lname, password from users where phone=$1",
            &[&body.username],
        )
        .await
        .map_err(internal_error)?;

    let user_id: i32 = row.get(0);
    let fname: String = row.get(1);
    let lname: String = row.get(2);
    let password: String = row.get(3);

    let is_valid_password = verify(&body.password, &password).map_err(internal_error)?;

    if !is_valid_password {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Invalid password".to_string(),
            })),
        ));
    }

    let user_response = json!({
        "user_id": user_id,
        "username": &body.username.clone(),
        "fname": fname.clone(),
        "lname": lname.clone(),
    });

    let claims = Claims {
        data: UserData {
            fname: fname.clone(),
            lname: lname.clone(),
            user_id: user_id,
        },
        exp: 3000,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )
    .map_err(internal_error)?;

    let cookie = CookieBuilder::new("Authorization", token)
        .path("/")
        .secure(true)
        .http_only(true)
        .max_age(Duration::days(3))
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
    _: Session,
    Json(body): Json<SignupForm>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let conn = pool.pg_pool.get().await.map_err(internal_error)?;

    let hashed_password = hash(&body.password, DEFAULT_COST).map_err(internal_error)?;
    let _ = conn
        .query(
            "INSERT INTO users(username, password, fname, lname) values($1, $2, $3, $4)",
            &[&body.username, &hashed_password, &body.fname, &body.lname],
        )
        .await
        .map_err(internal_error)?;

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
    if let Some(authorization_header) = header.get("authorization") {
        if let Ok(auth_str) = authorization_header.to_str() {
            let parts: Vec<&str> = auth_str.split_whitespace().collect();
            if parts.len() == 2 && parts[0].eq_ignore_ascii_case("Bearer") {
                return Some(parts[1].to_string());
            }
        }
    }
    None
}

pub async fn get_user_session(
    header: http::HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let secret_key = std::env::var("SECRET_KEY").unwrap();

    if let Some(token) = extract_authorization(&header).await {
        let token = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret_key.as_ref()),
            &Validation::default(),
        )
        .map_err(internal_error)?;
        let claims = token.claims;
        return Ok((
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            // header_map,
            Json(claims.data),
        ));
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Invalid token".to_string(),
            })),
        ))
    }
}
