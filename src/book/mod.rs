use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse, Json,
};
use serde_json::json;
use crate::AppState;
use tower_sessions::Session;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateBook {
    title: String,
    body: String,
    author_id: i32
}

pub async fn create_book(
	State(pool): State<AppState>,
	_: Session,
    Json(body): Json<CreateBook>,
) 
-> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)>
{
	let conn = pool.pg_pool.get().await.map_err(internal_error)?;
	let row = conn
        .query_one("INSERT INTO books(title, body, author_id) VALUES($1, $2, $3) RETURNING book_id", &[&body.title, &body.body, &body.author_id])
        .await
        .map_err(internal_error)?;
	let book_id: i32 = row.get(0);

    let _ = conn
        .query_one("INSERT INTO book(book_id, title, body) VALUES($1, $2, $3)", &[&book_id, &body.title, &body.body])
        .await
        .map_err(internal_error)?;

	let new_book = json!({
		"book_id": book_id,
		"title": &body.title.clone(),
		"body": &body.body.clone(),
		"author_id": &body.author_id.clone(),
	});

	Ok((
		StatusCode::OK,
		[(header::CONTENT_TYPE, "application/json")],
		Json(new_book),
	))
}

fn internal_error<E>(err: E) -> (StatusCode, Json<serde_json::Value>)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
		"message": err.to_string(),
	})))
}
