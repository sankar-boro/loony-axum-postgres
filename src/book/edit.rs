use crate::error::AppError;
use crate::traits::{Images, MoveImages};
use crate::utils::GetUserId;
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
pub struct EditBook {
    book_id: i32,
    title: String,
    body: String,
    images: Vec<Images>,
    theme: i16,
}

// @Edit
pub async fn edit_book(
    session: Session,
    State(pool): State<AppState>,
    Json(body): Json<EditBook>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = session.get_user_id().await?;
    let mut conn = pool.pg_pool.get().await?;
    let images = &serde_json::to_string(&body.images).unwrap();
    let _ = &body.images.move_images(
        &pool.dirs.tmp_upload,
        &pool.dirs.book_upload,
        user_id,
        body.book_id,
    );
    let state1 = conn
        .prepare("UPDATE books SET title=$1, body=$2, images=$3, theme=$4 WHERE book_id=$5")
        .await?;
    let state2 = conn
        .prepare("UPDATE book SET title=$1, body=$2, images=$3, theme=$4 WHERE book_id=$5")
        .await?;
    let transaction = conn.transaction().await?;
    transaction
        .execute(
            &state1,
            &[&body.title, &body.body, &images, &body.theme, &body.book_id],
        )
        .await?;
    transaction
        .execute(
            &state2,
            &[&body.title, &body.body, &images, &body.theme, &body.book_id],
        )
        .await?;
    transaction.commit().await?;

    let edit_book = json!({
        "book_id": &body.book_id,
        "title": &body.title.clone(),
        "body": &body.body.clone(),
        "book_id": &body.book_id,
        "images": &images,
        "theme": &body.theme,
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(edit_book),
    ))
}

#[derive(Deserialize, Serialize)]
pub struct EditBookNode {
    uid: i32,
    title: String,
    body: String,
    identity: i16,
    book_id: i32,
    images: Vec<Images>,
    theme: i16,
}

pub async fn edit_book_node(
    session: Session,
    State(pool): State<AppState>,
    Json(body): Json<EditBookNode>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = session.get_user_id().await?;
    let conn = pool.pg_pool.get().await?;
    let images = &serde_json::to_string(&body.images).unwrap();
    let _ = &body.images.move_images(
        &pool.dirs.tmp_upload,
        &pool.dirs.book_upload,
        user_id,
        body.book_id,
    );
    let _ = conn
        .execute(
            "UPDATE book SET title=$1, body=$2, images=$3, theme=$4 WHERE uid=$5",
            &[&body.title, &body.body, &images, &body.theme, &body.uid],
        )
        .await?;
    let edit_book = json!({
        "data": {
            "title": &body.title.clone(),
            "body": &body.body.clone(),
            "images": &images,
            "theme": &body.theme,
        }
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(edit_book),
    ))
}

// @End Edit
