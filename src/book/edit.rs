use crate::error::AppError;
use crate::traits::{move_images_to_s3, Images};
use crate::AppState;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize)]
pub struct EditBook {
    doc_id: i32,
    uid: i32,
    title: String,
    content: String,
    images: Vec<Images>
}

// @Edit
pub async fn edit_book(
    axum::extract::Extension(crate::utils::UserId(user_id)): axum::extract::Extension<crate::utils::UserId>,
    State(pool): State<AppState>,
    Json(body): Json<EditBook>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = pool.pg_pool.conn.get().await?;
    let images = &serde_json::to_string(&body.images).unwrap();
    let _ = move_images_to_s3(&body.images, pool.s3(), "tmp", "book", user_id, body.doc_id).await;
    let state1 = conn
        .prepare("UPDATE books SET title=$1, content=$2, images=$3 WHERE uid=$4 AND user_id=$5")
        .await?;
    let state2 = conn
        .prepare("UPDATE book SET title=$1, content=$2, images=$3 WHERE uid=$4 AND user_id=$5")
        .await?;
    let transaction = conn.transaction().await?;
    transaction
        .execute(
            &state1,
            &[&body.title, &body.content, &images, &body.doc_id, &user_id],
        )
        .await?;
    transaction
        .execute(
            &state2,
            &[&body.title, &body.content, &images, &body.uid, &user_id],
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
    axum::extract::Extension(crate::utils::UserId(user_id)): axum::extract::Extension<crate::utils::UserId>,
    State(pool): State<AppState>,
    Json(body): Json<EditBookNode>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.conn.get().await?;
    let images = &serde_json::to_string(&body.images).unwrap();
    let _ = move_images_to_s3(&body.images, pool.s3(), "tmp", "book", user_id, body.doc_id).await;
    if body.images.len() > 0 {
        let _ = conn
            .execute(
                "UPDATE book SET title=$1, content=$2, images=$3 WHERE uid=$4 AND user_id=$5",
                &[&body.title, &body.content, &images, &body.uid, &user_id],
            )
            .await?;
    }

    if body.images.len() == 0 {
        let _ = conn
            .execute(
                "UPDATE book SET title=$1, content=$2 WHERE uid=$3 AND user_id=$4",
                &[&body.title, &body.content, &body.uid, &user_id],
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
