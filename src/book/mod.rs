use crate::AppState;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_sessions::Session;

#[derive(Deserialize, Serialize)]
pub struct CreateBook {
    title: String,
    body: String,
    images: String,
    author_id: i32,
    password: String,
}

#[derive(Deserialize, Serialize)]
pub struct EditBook {
    book_id: i32,
    title: String,
    body: String,
    password: String,
}

#[derive(Deserialize, Serialize)]
pub struct GetBook {
    book_id: i32,
    title: String,
    body: String,
    images: String,
}

pub async fn create_book(
    State(pool): State<AppState>,
    _: Session,
    Json(body): Json<CreateBook>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let password = std::env::var("PASSWORD").unwrap();
    if &password != &body.password {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "message": "UNAUTHORIZED".to_string(),
            })),
        ));
    }
    let conn = pool.pg_pool.get().await.map_err(internal_error)?;
    let row = conn
        .query_one(
            "INSERT INTO books(title, body, images, author_id) VALUES($1, $2, $3, $4) RETURNING book_id",
            &[&body.title, &body.body, &body.images, &body.author_id],
        )
        .await
        .map_err(internal_error)?;

    let book_id: i32 = row.get(0);

    let _ = conn
        .query_one(
            "INSERT INTO book(book_id, title, body, images) VALUES($1, $2, $3, $4) RETURNING *",
            &[&book_id, &body.title, &body.body, &body.images],
        )
        .await
        .map_err(internal_error)?;

    let new_book = json!({
        "book_id": book_id,
        "title": &body.title.clone(),
        "body": &body.body.clone(),
        "images": &body.images.clone(),
        "author_id": &body.author_id,
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(new_book),
    ))
}

pub async fn edit_book(
    State(pool): State<AppState>,
    _: Session,
    Json(body): Json<EditBook>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let password = std::env::var("PASSWORD").unwrap();
    if &password != &body.password {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "message": "UNAUTHORIZED".to_string(),
            })),
        ));
    }
    let conn = pool.pg_pool.get().await.map_err(internal_error)?;
    let row = conn
        .query_one(
            "UPDATE books SET title=$1 AND body=$2 WHERE book_id=$3",
            &[&body.title, &body.body, &body.book_id],
        )
        .await
        .map_err(internal_error)?;

    let book_id: i32 = row.get(0);

    let _ = conn
        .query_one(
            "UPDATE book SET title=$1 AND body=$2 WHERE book_id=$3",
            &[&book_id, &body.title, &body.body, &body.book_id],
        )
        .await
        .map_err(internal_error)?;

    let edit_book = json!({
        "book_id": book_id,
        "title": &body.title.clone(),
        "body": &body.body.clone(),
        "book_id": &body.book_id
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(edit_book),
    ))
}

#[derive(Deserialize, Serialize)]
pub struct DeleteBook {
    book_id: i32,
}

pub async fn delete_book(
    State(pool): State<AppState>,
    _: Session,
    Json(body): Json<DeleteBook>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let conn = pool.pg_pool.get().await.map_err(internal_error)?;
    let row = conn
        .query_one("DELETE FROM books WHERE book_id=$1", &[&body.book_id])
        .await
        .map_err(internal_error)?;

    if row.len() == 0 {
        return Ok((
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            Json(json!({
                "data": "Could not delete book"
            })),
        ));
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "data": "book deleted"
        })),
    ))
}

pub async fn get_all_books(
    State(pool): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let conn = pool.pg_pool.get().await.map_err(internal_error)?;
    let rows = conn
        .query("SELECT book_id, title, body, images FROM books", &[])
        .await
        .map_err(internal_error)?;

    let mut books: Vec<GetBook> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        let book_id: i32 = rows[index].get(0);
        let title: String = rows[index].get(1);
        let body: String = rows[index].get(2);
        let images: String = rows[index].get(3);
        books.push(GetBook {
            book_id,
            title,
            body,
            images,
        })
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "data": books
        })),
    ))
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
