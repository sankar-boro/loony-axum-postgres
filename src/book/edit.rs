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
    doc_id: i32,
    uid: i32,
    title: String,
    content: String,
}

// @Edit
pub async fn edit_book(
    // session: Session,
    State(pool): State<AppState>,
    Json(body): Json<EditBook>,
) -> Result<impl IntoResponse, AppError> {
    // let user_id = session.get_user_id().await?;
    let mut conn = pool.pg_pool.get().await?;
    // let images = &serde_json::to_string(&body.images).unwrap();
    // let _ = &body.images.move_images(
    //     &pool.dirs.tmp_upload,
    //     &pool.dirs.book_upload,
    //     user_id,
    //     body.doc_id,
    // );
    let state1 = conn
        .prepare("UPDATE books SET title=$1, content=$2 WHERE uid=$3")
        .await?;
    let state2 = conn
        .prepare("UPDATE book SET title=$1, content=$2 WHERE uid=$3")
        .await?;
    let transaction = conn.transaction().await?;
    transaction
        .execute(
            &state1,
            &[&body.title, &body.content, &body.doc_id],
        )
        .await?;
    transaction
        .execute(
            &state2,
            &[&body.title, &body.content, &body.uid],
        )
        .await?;
    transaction.commit().await?;

    let edit_book = json!({
        "doc_id": &body.doc_id,
        "title": &body.title,
        "content": &body.content,
        "uid": &body.uid
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
    content: String,
    identity: i16,
    doc_id: i32,
    images: Vec<Images>,
}

/// # Returns
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
        body.doc_id,
    );
    if body.images.len() > 0 {
        let _ = conn
            .execute(
                "UPDATE book SET title=$1, content=$2, images=$3 WHERE uid=$4",
                &[&body.title, &body.content, &images, &body.uid],
            )
            .await?;
    }

    if body.images.len() == 0 {
        let _ = conn
            .execute(
                "UPDATE book SET title=$1, content=$2 WHERE uid=$3",
                &[&body.title, &body.content, &body.uid],
            )
            .await?;
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(body),
    ))
}

// @End Edit
