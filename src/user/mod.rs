use crate::error::Error;

use actix_session::Session;
use deadpool_postgres::Pool;
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct GetUserX {
	email: String,
}

static GET_USERX: &str = "SELECT userId, fname, lname, password FROM users WHERE email=$1";

pub async fn get_user(
	request: web::Json<GetUserX>, 
	app: web::Data<Pool>
) 
-> Result<HttpResponse, Error> 
{
	let client = app.get().await?;
    let stmt = client.prepare_cached(GET_USERX).await?;
    let rows = client.query(&stmt, &[&request.email]).await?;
	let user_id: i32 = rows[0].get(0);
	let fname: String = rows[0].get(1);
	let lname: String = rows[0].get(2);
	
	let auth_user_session = json!({
		"userId": user_id.clone(),
		"email": &request.email.clone(),
		"fname": fname.clone(),
		"lname": lname.clone(),
	});
	Ok(HttpResponse::Ok().json(auth_user_session))
}

pub async fn user_session(session: Session) 
-> Result<HttpResponse, actix_web::Error> {
    let auth_user_session = session.get::<String>("AUTH_USER")?;
    match auth_user_session {
        Some(session) => {
            Ok(HttpResponse::Ok().json(json!({
				"status": 200,
				"auth": true,
				"data": session
			})))
        }
        None => Err(Error::from("REQUEST_LOGIN").into())   
    }
}

pub async fn logout_user(session: Session) -> HttpResponse {
	session.clear();
	session.purge();
	HttpResponse::Ok().body("Logged out.")
  }