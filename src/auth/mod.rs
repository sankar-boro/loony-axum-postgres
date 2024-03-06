use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse, Json,
};
use serde_json::json;
use crate::AppState;
use serde::Deserialize;
use tower_sessions::Session;
use cookie::CookieBuilder;
use time::Duration;
use bcrypt::{verify, hash, DEFAULT_COST};

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
    lname: String
}


fn internal_error<E>(err: E) -> (StatusCode, Json<serde_json::Value>)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
		"message": err.to_string(),
	})))
}


pub async fn login(
	State(pool): State<AppState>,
	_: Session,
    Json(body): Json<LoginForm>,
) 
-> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)>
{
	// request.validate()?;
	let conn = pool.pg_pool.get().await.map_err(internal_error)?;
	let row = conn
        .query_one("select user_id, fname, lname, password from users where phone=$1", &[&body.username])
        .await
        .map_err(internal_error)?;
	
	let user_id: i32 = row.get(0);
	let fname: String = row.get(1);
	let lname: String = row.get(2);
	let password: String = row.get(3);

	let is_valid_password = verify(&body.password, &password).map_err(|err| { (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
		"message": err.to_string(),
	}))) })?;

	if !is_valid_password {
		return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
			"message": "Invalid password".to_string(),
		}))));
	}

	let user_response = json!({
		"userId": user_id,
		"phone": &body.username.clone(),
		"fname": fname.clone(),
		"lname": lname.clone(),
	});

    
	let cookie = CookieBuilder::new("Authorization", "sankar")
	.domain("http://localhost:3000")
	.path("/")
	.secure(true)  // Set to true for HTTPS only
	.http_only(true) // Set to true to prevent JavaScript access
	.max_age(Duration::seconds(30)) // Set cookie expiration
	.build().to_string();


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
) 
-> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)>
{
	// request.validate()?;
	let conn = pool.pg_pool.get().await.map_err(internal_error)?;

    let hashed_password= hash(&body.password, DEFAULT_COST).map_err(|err| {(StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
        "message": err.to_string(),
    }))) })?;
	let _ = conn
        .query("INSERT INTO users(username, password, fname, lname) values($1, $2, $3, $4)", &[&body.username, &hashed_password, &body.fname, &body.lname])
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


