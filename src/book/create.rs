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
use serde_json::{json, Value};
use tower_sessions::Session;

#[derive(Deserialize, Serialize)]
pub struct CreateBook {
    title: String,
    content: String,
    images: Vec<Images>,
    tags: Option<Vec<String>>,
}

// @Create

pub async fn create_book(
    session: Session,
    State(pool): State<AppState>,
    Json(body): Json<CreateBook>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = session.get_user_id().await?;
    let identity: i16 = 100;
    let images = &serde_json::to_string(&body.images).unwrap();

    let mut conn = pool.pg_pool.conn.get().await?;

    let insert_books_query = conn
        .prepare("INSERT INTO books(user_id, title, content, images) VALUES($1, $2, $3, $4) RETURNING uid")
        .await?;
    let insert_book_query = conn
        .prepare("INSERT INTO book(user_id, doc_id, title, content, identity, images) VALUES($1, $2, $3, $4, $5, $6) RETURNING uid")
        .await?;

    let transaction = conn.transaction().await?;

    let row = transaction
        .query_one(
            &insert_books_query,
            &[&user_id, &body.title, &body.content, &images],
        )
        .await?;

    let doc_id: i32 = row.get(0);

    transaction
        .execute(
            &insert_book_query,
            &[
                &user_id,
                &doc_id,
                &body.title,
                &body.content,
                &identity,
                &images,
            ],
        )
        .await?;

    transaction.commit().await?;

    // if let Some(tags) = body.tags {
    //     let mut all_tags: Vec<(i32, i32, &str, i32)> = Vec::new();
    //     tags.iter().for_each(|__tag| {
    //         all_tags.push((doc_id, user_id, __tag, 1));
    //     });
    
    //     conn.query(
    //         &insert_tags("book_tags", "(doc_id, user_id, tag, score)", all_tags),
    //         &[],
    //     )
    //     .await?;
    // }

    let _ = &body.images.move_images(
        &pool.get_tmp_path(),
        &pool.get_book_path(),
        user_id,
        doc_id,
    );

    let new_book = json!({
        "user_id": &user_id,
        "doc_id": doc_id,
        "title": &body.title,
        "content": &body.content,
        "identity": &identity,
        "images": &images
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(new_book),
    ))
}

#[derive(Deserialize, Serialize)]
pub struct AddBookNode {
    doc_id: i32,
    title: String,
    content: String,
    images: Vec<Images>,
    parent_id: i32,
    page_id: Option<i32>,
    identity: i16,
    tags: Option<Vec<String>>,
    parent_identity: i16,
}

pub async fn append_book_node(
    session: Session,
    State(pool): State<AppState>,
    Json(body): Json<AddBookNode>,
) -> Result<impl IntoResponse, AppError> {
    if body.parent_identity == 101 && body.identity == 103 {
        return Err(AppError::InternalServerError(String::from("Not Allowed")));
    }
    let user_id = session.get_user_id().await?;
    let doc_id = body.doc_id;
    let mut conn = pool.pg_pool.conn.get().await?;

    let images = &serde_json::to_string(&body.images).unwrap();

    let has_row_update = conn
        .query_one(
            "SELECT uid, parent_id, identity from book where parent_id=$1 AND identity=$2 AND deleted_at is NULL",
            &[&body.parent_id, &body.identity],
        )
        .await;

    let state1 = conn
    .prepare(
        "INSERT INTO book(user_id, doc_id, page_id, parent_id, title, content, identity, images) values($1, $2, $3, $4, $5, $6, $7, $8) returning uid",
    )
    .await?;
    let state2 = conn
        .prepare("UPDATE book SET parent_id=$1 where uid=$2 RETURNING uid")
        .await?;

    let transaction = conn.transaction().await?;
    let res1 = transaction
        .query_one(
            &state1,
            &[
                &user_id,
                &body.doc_id,
                &body.page_id,
                &body.parent_id,
                &body.title,
                &body.content,
                &body.identity,
                &images,
            ],
        )
        .await?;

    let new_node_id: i32 = res1.get(0);

    let mut update_node: Option<Value> = None;

    if let Ok(update_row) = has_row_update {
        if !update_row.is_empty() {
            let update_row_id: i32 = update_row.get(0);
            let identity: Option<i16> = update_row.get(2);

            if &body.identity >= &identity.unwrap() {
                transaction
                    .execute(&state2, &[&new_node_id, &update_row_id])
                    .await?;
                update_node = Some(json!({
                    "uid": update_row_id,
                    "parent_id": new_node_id
                }))
            }
        }
    }
    transaction.commit().await?;

    // if let Some(tags) = body.tags {
    //     let mut all_tags: Vec<(i32, i32, &str, i32)> = Vec::new();
    //     tags.iter().for_each(|__tag| {
    //         all_tags.push((doc_id, user_id, __tag, 1));
    //     });
    
    //     conn.query(
    //         &insert_tags("book_tags", "(doc_id, user_id, tag, score)", all_tags),
    //         &[],
    //     )
    //     .await?;
    // }

    let _ = &body.images.move_images(
        &pool.get_tmp_path(),
        &pool.get_book_path(),
        user_id,
        body.doc_id,
    );

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "new_node": {
                "uid": new_node_id,
                "parent_id": &body.parent_id,
                "title": &body.title,
                "content": &body.content,
                "images": &images,
                "identity": &body.identity,
                "page_id": &body.page_id
            },
            "update_node": update_node
        })),
    ))
}

// @End Create

