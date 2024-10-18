pub mod edit;
pub mod get;

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
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_sessions::Session;

#[derive(Deserialize, Serialize)]
pub struct CreateBook {
    title: String,
    body: String,
    images: Vec<Images>,
    tags: Option<Vec<String>>,
    theme: i16,
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

    let mut conn = pool.pg_pool.get().await?;

    let insert_books_query = conn
        .prepare("INSERT INTO books(title, body, images, user_id, theme) VALUES($1, $2, $3, $4, $5) RETURNING uid")
        .await?;
    let insert_book_query = conn
        .prepare("INSERT INTO book(book_id, title, identity, body, images, user_id, theme) VALUES($1, $2, $3, $4, $5, $6, $7) RETURNING uid")
        .await?;
    let mut insert_tags_query: Option<String> = None;
    if let Some(tags) = &body.tags {
        insert_tags_query = Some(format!(
            "WITH ins AS (
                INSERT INTO tags (name)
                VALUES {}
                ON CONFLICT (name) DO NOTHING
                RETURNING uid, name
            )
            SELECT uid, name FROM ins
            UNION ALL
            SELECT uid, name FROM tags WHERE name IN ({}) AND NOT EXISTS (SELECT uid, name FROM ins)",
            tags.iter()
                .map(|s| format!("('{}')", s))
                .collect::<Vec<String>>()
                .join(", "),
                tags.iter()
                .map(|s| format!("'{}'", s))
                .collect::<Vec<String>>()
                .join(", ")
        ));
    }
    let transaction = conn.transaction().await?;

    let row = transaction
        .query_one(
            &insert_books_query,
            &[&body.title, &body.body, &images, &user_id, &body.theme],
        )
        .await?;

    let book_id: i32 = row.get(0);

    if let Some(insert_tags_query) = insert_tags_query {
        let res = transaction.query(&insert_tags_query, &[]).await?;
        let mut tag_rows: Vec<(i32, i32, i32)> = Vec::new();
        for row in res.iter() {
            tag_rows.push((row.get(0), book_id, user_id));
        }
        let book_tag_query = format!(
            "INSERT INTO book_tags(tag_id, book_id) VALUES {} RETURNING uid",
            tag_rows
                .iter()
                .map(|(tid, bid, _)| format!("('{}', '{}')", tid, bid))
                .collect::<Vec<String>>()
                .join(", "),
        );
        let user_tag_query = format!(
            "INSERT INTO user_tags(tag_id, user_id) VALUES {} RETURNING uid",
            tag_rows
                .iter()
                .map(|(tid, _, uid)| format!("('{}', '{}')", tid, uid))
                .collect::<Vec<String>>()
                .join(", "),
        );
        transaction.execute(&book_tag_query, &[]).await?;
        transaction.execute(&user_tag_query, &[]).await?;
    }

    transaction
        .execute(
            &insert_book_query,
            &[
                &book_id,
                &body.title,
                &identity,
                &body.body,
                &images,
                &user_id,
                &body.theme,
            ],
        )
        .await?;
    transaction.commit().await?;

    let _ = &body.images.move_images(
        &pool.dirs.tmp_upload,
        &pool.dirs.book_upload,
        user_id,
        book_id,
    );

    let new_book = json!({
        "book_id": book_id,
        "title": &body.title.clone(),
        "body": &body.body.clone(),
        "identity": &identity,
        "images": &images,
        "user_id": &user_id,
        "theme": &body.theme
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(new_book),
    ))
}

#[derive(Deserialize, Serialize)]
pub struct AddBookNode {
    book_id: i32,
    title: String,
    body: String,
    images: Vec<Images>,
    parent_id: i32,
    page_id: Option<i32>,
    identity: i16,
    tags: Option<String>,
    theme: i16,
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
    let mut conn = pool.pg_pool.get().await?;

    let images = &serde_json::to_string(&body.images).unwrap();

    let has_row_update = conn
        .query_one(
            "SELECT uid, parent_id, identity from book where parent_id=$1 AND identity=$2 AND deleted_at is NULL",
            &[&body.parent_id, &body.identity],
        )
        .await;

    let state1 = conn
    .prepare(
        "INSERT INTO book(book_id, page_id, parent_id, title, body, identity, images, theme) values($1, $2, $3, $4, $5, $6, $7, $8) returning uid",
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
                &body.book_id,
                &body.page_id,
                &body.parent_id,
                &body.title,
                &body.body,
                &body.identity,
                &images,
                &body.theme,
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

    let _ = &body.images.move_images(
        &pool.dirs.tmp_upload,
        &pool.dirs.book_upload,
        user_id,
        body.book_id,
    );

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "new_node": {
                "uid": new_node_id,
                "parent_id": &body.parent_id,
                "title": &body.title,
                "body": &body.body,
                "images": &images,
                "identity": &body.identity,
                "page_id": &body.page_id,
                "theme": &body.theme
            },
            "update_node": update_node
        })),
    ))
}

// @End Create

// @Delete

#[derive(Deserialize, Serialize)]
pub struct DeleteBook {
    book_id: i32,
}

pub async fn delete_book(
    State(pool): State<AppState>,
    Json(body): Json<DeleteBook>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = pool.pg_pool.get().await?;
    let current_time = Local::now();

    let state1 = conn
        .prepare("UPDATE book SET deleted_at=$1 WHERE book_id=$2")
        .await?;
    let state2 = conn
        .prepare("UPDATE books SET deleted_at=$1 WHERE book_id=$2")
        .await?;
    let transaction = conn.transaction().await?;
    transaction
        .execute(&state1, &[&current_time, &body.book_id])
        .await?;
    transaction
        .execute(&state2, &[&current_time, &body.book_id])
        .await?;
    transaction.commit().await?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "data": "book deleted"
        })),
    ))
}

#[derive(Deserialize, Serialize)]
pub struct DeleteBookNode {
    identity: i16,
    delete_id: i32,
    parent_id: i32,
}

pub async fn delete_book_node(
    State(pool): State<AppState>,
    Json(body): Json<DeleteBookNode>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = pool.pg_pool.get().await?;

    // Prepare to find ids to delete
    // Applies only for nodes where identity is 101, 102
    let mut delete_row_ids: Vec<i32> = Vec::new();
    delete_row_ids.push(body.delete_id);

    let delete_rows = conn
        .query("SELECT uid FROM book where page_id=$1", &[&body.delete_id])
        .await?;

    if delete_rows.len() > 0 {
        for row in delete_rows.iter() {
            let uid = row.get(0);
            delete_row_ids.push(uid);
        }
    }

    if body.identity == 101 {
        let sub_section_nodes = conn
            .query(
                "SELECT uid FROM book where page_id=ANY($1)",
                &[&delete_row_ids],
            )
            .await?;

        if sub_section_nodes.len() > 0 {
            for sub_section in sub_section_nodes.iter() {
                let uid2 = sub_section.get(0);
                delete_row_ids.push(uid2);
            }
        }
    }
    // Check if there is a node to update
    let update_row_exist = conn
        .query_opt(
            "SELECT uid, parent_id from book where parent_id=$1 AND identity=$2 AND deleted_at IS NULL",
            &[&body.delete_id, &body.identity],
        )
        .await?;
    let current_time = Local::now();
    let state1 = conn
        .prepare("UPDATE book set deleted_at=$1 WHERE uid=ANY($2)")
        .await?;
    let update_bot_node_query = conn
        .prepare("UPDATE book SET parent_id=$1 WHERE uid=$2")
        .await?;
    let transaction = conn.transaction().await?;
    let num_deleted_rows = transaction
        .execute(&state1, &[&current_time, &delete_row_ids])
        .await?;

    let mut updated_id: Option<i32> = None;
    if let Some(update_row) = update_row_exist {
        let update_id: i32 = update_row.get(0);
        transaction
            .execute(&update_bot_node_query, &[&body.parent_id, &update_id])
            .await?;
        updated_id = Some(update_id);
    }
    transaction.commit().await?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "delete_nodes": delete_row_ids,
            "update_node": {
                "uid": updated_id,
                "parent_id": &body.parent_id,
            },
            "rows": num_deleted_rows
        })),
    ))
}

// @End Delete

pub async fn test_query(State(pool): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let page_ids: Vec<i32> = Vec::from([1, 2]);
    let rows = conn
        .query("SELECT uid FROM book where page_id = ANY($1)", &[&page_ids])
        .await?;

    for i in rows.iter() {
        let uid: i32 = i.get(0);
        println!("{}", uid);
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "data": "book deleted"
        })),
    ))
}
