use crate::error::AppError;
use crate::traits::{Images, MoveImages};
use crate::utils::GetUserId;
use crate::AppState;
use axum::{
    extract::{Query, State},
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
    tags: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct EditBook {
    book_id: i32,
    title: String,
    body: String,
    images: Vec<Images>,
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

    let state1 = conn
        .prepare("INSERT INTO books(title, body, images, user_id, tags) VALUES($1, $2, $3, $4, $5) RETURNING book_id")
        .await?;
    let state2 = conn
        .prepare("INSERT INTO book(book_id, title, identity, body, images, user_id, tags) VALUES($1, $2, $3, $4, $5, $6, $7) RETURNING *")
        .await?;
    let transaction = conn.transaction().await?;

    let row = transaction
        .query_one(
            &state1,
            &[&body.title, &body.body, &images, &user_id, &body.tags],
        )
        .await?;

    let book_id: i32 = row.get(0);

    transaction
        .execute(
            &state2,
            &[
                &book_id,
                &body.title,
                &identity,
                &body.body,
                &images,
                &user_id,
                &body.tags,
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
}

pub async fn append_book_node(
    session: Session,
    State(pool): State<AppState>,
    Json(body): Json<AddBookNode>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = session.get_user_id().await?;
    let mut conn = pool.pg_pool.get().await?;

    let images = &serde_json::to_string(&body.images).unwrap();

    let has_row_update = conn
        .query_one(
            "SELECT uid, parent_id, identity from book where parent_id=$1 AND identity=$2",
            &[&body.parent_id, &body.identity],
        )
        .await;

    let state1 = conn
    .prepare(
        "INSERT INTO book(book_id, page_id, parent_id, title, body, identity, images, tags) values($1, $2, $3, $4, $5, $6, $7, $8) returning uid",
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
                &body.tags,
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
                    "parent_id": new_node_id,
                    "description": "update parent_id where uid"
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
                "tags": &body.tags
            },
            "update_node": update_node
        })),
    ))
}

// @End Create

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
        .prepare("UPDATE books SET title=$1, body=$2, images=$3 WHERE book_id=$4")
        .await?;
    let state2 = conn
        .prepare("UPDATE book SET title=$1, body=$2, images=$3 WHERE book_id=$4")
        .await?;
    let transaction = conn.transaction().await?;
    transaction
        .execute(&state1, &[&body.title, &body.body, &images, &body.book_id])
        .await?;
    transaction
        .execute(&state2, &[&body.title, &body.body, &images, &body.book_id])
        .await?;
    transaction.commit().await?;

    let edit_book = json!({
        "book_id": &body.book_id,
        "title": &body.title.clone(),
        "body": &body.body.clone(),
        "book_id": &body.book_id,
        "images": &images,
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
            "UPDATE book SET title=$1, body=$2, images=$3 WHERE uid=$4",
            &[&body.title, &body.body, &images, &body.uid],
        )
        .await?;
    let edit_book = json!({
        "data": {
            "title": &body.title.clone(),
            "body": &body.body.clone(),
            "images": &images,
        }
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(edit_book),
    ))
}

// @End Edit

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
            "SELECT uid, parent_id from book where parent_id=$1 AND identity=$2",
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
            "updated_id": updated_id,
            "deleted_ids": delete_row_ids,
            "parent_id": &body.parent_id,
            "num_deleted_rows": num_deleted_rows
        })),
    ))
}

// @End Delete

// @Get

#[derive(Deserialize, Serialize)]
pub struct GetHomeBooks {
    book_id: i32,
    title: String,
    body: String,
    images: String,
}

pub async fn get_all_books(State(pool): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query(
            "SELECT book_id, title, body, images FROM books where deleted_at is NULL",
            &[],
        )
        .await?;

    let mut books: Vec<GetHomeBooks> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        let book_id: i32 = rows[index].get(0);
        let title: String = rows[index].get(1);
        let body: String = rows[index].get(2);
        let images: String = rows[index].get(3);
        books.push(GetHomeBooks {
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

#[derive(Deserialize, Serialize)]
pub struct ChaptersByBookId {
    uid: i32,
    parent_id: Option<i32>,
    title: String,
    body: String,
    images: Option<String>,
    identity: i16,
    page_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct ChaptersByBookIdRequest {
    book_id: i32,
}

pub async fn get_book_chapters(
    State(pool): State<AppState>,
    query: Query<ChaptersByBookIdRequest>,
) -> Result<impl IntoResponse, AppError> {
    let book_info: ChaptersByBookIdRequest = query.0;

    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query(
            "SELECT uid, parent_id, title, body, images, identity, page_id FROM book where book_id=$1 AND identity<=101 AND deleted_at is null",
            &[&book_info.book_id],
        )
        .await?;

    let mut books: Vec<ChaptersByBookId> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        let uid: i32 = rows[index].get(0);
        let parent_id: Option<i32> = rows[index].get(1);
        let title: String = rows[index].get(2);
        let body: String = rows[index].get(3);
        let images: Option<String> = rows[index].get(4);
        let identity: i16 = rows[index].get(5);
        let page_id: Option<i32> = rows[index].get(6);
        books.push(ChaptersByBookId {
            uid,
            parent_id,
            title,
            body,
            images,
            identity,
            page_id,
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

#[derive(Deserialize)]
pub struct BookByPageId {
    book_id: i32,
    page_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct BookNodesByPageId {
    uid: i32,
    parent_id: Option<i32>,
    title: String,
    body: String,
    images: Option<String>,
    identity: i16,
    page_id: Option<i32>,
}

pub async fn get_book_sections(
    State(pool): State<AppState>,
    query: Query<BookByPageId>,
) -> Result<impl IntoResponse, AppError> {
    let book_info: BookByPageId = query.0;

    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query(
            "SELECT uid, parent_id, title, body, images, identity, page_id FROM book where book_id=$1 AND page_id=$2 AND identity=102 and deleted_at is null",
            &[&book_info.book_id, &book_info.page_id],
        )
        .await?;

    let mut books: Vec<BookNodesByPageId> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        let uid: i32 = rows[index].get(0);
        let parent_id: Option<i32> = rows[index].get(1);
        let title: String = rows[index].get(2);
        let body: String = rows[index].get(3);
        let images: Option<String> = rows[index].get(4);
        let identity: i16 = rows[index].get(5);
        let page_id: Option<i32> = rows[index].get(6);
        books.push(BookNodesByPageId {
            uid,
            parent_id,
            title,
            body,
            images,
            identity,
            page_id,
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

#[derive(Deserialize)]
pub struct BookBySectionId {
    book_id: i32,
    page_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct BookNodesBySectionId {
    uid: i32,
    parent_id: Option<i32>,
    title: String,
    body: String,
    images: Option<String>,
    identity: i16,
    page_id: Option<i32>,
}

pub async fn get_book_sub_sections(
    State(pool): State<AppState>,
    query: Query<BookBySectionId>,
) -> Result<impl IntoResponse, AppError> {
    let book_info: BookBySectionId = query.0;

    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query(
            "SELECT uid, parent_id, title, body, images, identity, page_id FROM book where book_id=$1 AND page_id=$2 AND identity=103 and deleted_at is null",
            &[&book_info.book_id, &book_info.page_id],
        )
        .await?;

    let mut books: Vec<BookNodesBySectionId> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        let uid: i32 = rows[index].get(0);
        let parent_id: Option<i32> = rows[index].get(1);
        let title: String = rows[index].get(2);
        let body: String = rows[index].get(3);
        let images: Option<String> = rows[index].get(4);
        let identity: i16 = rows[index].get(5);
        let page_id: Option<i32> = rows[index].get(6);
        books.push(BookNodesBySectionId {
            uid,
            parent_id,
            title,
            body,
            images,
            identity,
            page_id,
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

// @End Get

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
