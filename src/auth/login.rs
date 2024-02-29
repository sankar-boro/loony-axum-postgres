use std::sync::Arc;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::State,
    http::{header, Response, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
// use axum_extra::extract::cookie::{Cookie, SameSite};
// use jsonwebtoken::{encode, EncodingKey, Header};
// use rand_core::OsRng;
use serde_json::json;
use crate::AppState;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct LoginForm {
	email: String,
	password: String,
}

// #[derive(Serialize, Debug)]
pub struct GetUser {
	userId: i32,
	email: String,
	password: String,
	fname: String,
	lname: String,
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

pub async fn login(
	State(pool): State<AppState>,
    Json(body): Json<LoginForm>,
) 
-> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)>
{
	
	// request.validate()?;
	let conn = pool.pg_pool.get().await.map_err(internal_error);
	if let Ok(conn) = conn {

        let row = conn
        .query_one("select user_id, fname, lname from users where email=$1", &[&body.email])
        .await
        .map_err(internal_error).unwrap();
		let user_id: i32 = row.get(0);
		let fname: String = row.get(1);
		let lname: String = row.get(2);
		let password: String = row.get(3);
		let password: Vec<u8> = password.as_bytes().to_vec();
		let user_response = json!({
			"userId": user_id,
			"email": &body.email.clone(),
			"fname": fname.clone(),
			"lname": lname.clone(),
		});
		return Ok(Json(user_response));
	}

	Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
		"message": "User not found",
	}))))


	// validate_user_credentials(&request.password, &password)?;
	

	// let mut locked_session = app.session.lock().unwrap();

	// let auth_user_session = auth_user_session.clone().to_string();
	
	// locked_session.hset(&request.email, "AUTH_USER", auth_user_session.clone()).unwrap_or_else(|_| ());
	// locked_session.hset(&request.email, "AUTH_ID", user_id).unwrap_or_else(|_| ());

	// Ok(HttpResponse::Ok()
    //     .cookie(cookie::Cookie::build("Authorization", "sankar")
    //         .http_only(true)
    //         .max_age(Duration::hours(3))
    //         .same_site(cookie::SameSite::None)
    //         .secure(true)
    //         .finish())
    //     .json(auth_user_session))
}


