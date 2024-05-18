use crate::error::AppError;
use crate::traits::{Images, MoveImages};
use crate::AppState;
use crate::{delete_nodes_query, delete_where, update_query};
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize)]
pub struct CreateBook {
    title: String,
    body: String,
    images: Vec<Images>,
    author_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct EditBook {
    book_id: i32,
    title: String,
    body: String,
    images: Vec<Images>,
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
    Json(body): Json<CreateBook>,
) -> Result<impl IntoResponse, AppError> {
    let identity: i16 = 100;
    let conn = pool.pg_pool.get().await?;
    let images = &serde_json::to_string(&body.images).unwrap();
    let _ = &body
        .images
        .move_images(&pool.dirs.file_upload_tmp, &pool.dirs.file_upload);
    let row = conn
        .query_one(
            "INSERT INTO books(title, body, images, author_id) VALUES($1, $2, $3, $4) RETURNING book_id",
            &[&body.title, &body.body, &images, &body.author_id],
        )
        .await?;

    let book_id: i32 = row.get(0);

    let _ = conn
        .query_one(
            "INSERT INTO book(book_id, title, identity, body, images) VALUES($1, $2, $3, $4, $5) RETURNING *",
            &[&book_id, &body.title, &identity, &body.body, &images],
        )
        .await?;

    let new_book = json!({
        "book_id": book_id,
        "title": &body.title.clone(),
        "body": &body.body.clone(),
        "identity": &identity,
        "images": &images,
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
    Json(body): Json<EditBook>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = pool.pg_pool.get().await?;
    let images = &serde_json::to_string(&body.images).unwrap();
    let _ = &body
        .images
        .move_images(&pool.dirs.file_upload_tmp, &pool.dirs.file_upload);
    let rows1 = conn
        .prepare("UPDATE books SET title=$1, body=$2, $images=$3 WHERE book_id=$4")
        .await?;
    let rows2 = conn
        .prepare("UPDATE book SET title=$1, body=$2, $images=$3 WHERE book_id=$4")
        .await?;
    let transaction = conn.transaction().await?;
    transaction
        .execute(&rows1, &[&body.title, &body.body, &images, &body.book_id])
        .await?;
    transaction
        .execute(&rows2, &[&body.title, &body.body, &images, &body.book_id])
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
    State(pool): State<AppState>,
    Json(body): Json<EditBookNode>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let images = &serde_json::to_string(&body.images).unwrap();
    let _ = &body
        .images
        .move_images(&pool.dirs.file_upload_tmp, &pool.dirs.file_upload);
    let _ = conn
        .execute(
            "UPDATE book SET title=$1, body=$2, images=$3 WHERE uid=$4",
            &[&body.title, &body.body, &images, &body.uid],
        )
        .await?;

    if *&body.identity == 100 {
        let _ = conn
            .execute(
                "UPDATE books SET title=$1, body=$2, images=$3 WHERE book_id=$4",
                &[&body.title, &body.body, &images, &body.book_id],
            )
            .await?;
    }

    let edit_book = json!({
        "title": &body.title.clone(),
        "body": &body.body.clone(),
        "images": &images,
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(edit_book),
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
}

pub async fn append_book_node(
    State(pool): State<AppState>,
    Json(body): Json<AddBookNode>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;

    let update_row = conn
        .query_one(
            "SELECT uid, parent_id, identity from book where parent_id=$1 AND identity=$2",
            &[&body.parent_id, &body.identity],
        )
        .await;
    let images = &serde_json::to_string(&body.images).unwrap();
    let _ = &body
        .images
        .move_images(&pool.dirs.file_upload_tmp, &pool.dirs.file_upload);
    let new_node = conn
    .query_one(
            "INSERT INTO book(book_id, page_id, parent_id, title, body, identity, images) values($1, $2, $3, $4, $5, $6, $7) returning uid",
            &[&body.book_id, &body.page_id, &body.parent_id, &body.title, &body.body, &body.identity, &images],
        )
        .await?;

    let new_node_uid: i32 = new_node.get(0);
    let mut update_row_uid: Option<i32> = None;
    if let Ok(update_row) = update_row {
        if !update_row.is_empty() {
            update_row_uid = update_row.get(0);
            let identity: Option<i16> = update_row.get(2);
            if &body.identity >= &identity.unwrap() {
                conn.query_one(
                    "UPDATE book SET parent_id=$1 where uid=$2 RETURNING uid",
                    &[&new_node_uid, &update_row_uid],
                )
                .await?;
            }
        }
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "new_node": {
                "uid": new_node_uid,
                "parent_id": &body.parent_id,
                "title": &body.title,
                "body": &body.body,
                "images": &images,
                "identity": &body.identity,
                "page_id": &body.page_id
            },
            "update_node": {
                "update_row_id": update_row_uid,
                "update_row_parent_id": new_node_uid
            }
        })),
    ))
}

#[derive(Deserialize, Serialize)]
pub struct DeleteBook {
    book_id: i32,
}

pub async fn delete_book(
    State(pool): State<AppState>,
    Json(body): Json<DeleteBook>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;

    let delete_books = delete_where!("books", "book_id", &body.book_id);
    let delete_book = delete_where!("book", "book_id", &body.book_id);
    conn.batch_execute(&format!("{}; {}", &delete_books, &delete_book))
        .await?;

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
    delete_node_id: i32,
    update_parent_id: i32,
}

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

pub async fn delete_book_node(
    State(pool): State<AppState>,
    Json(body): Json<DeleteBookNode>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;

    let mut delete_row_ids: Vec<i32> = Vec::new();
    let delete_rows = conn
        .query(
            "SELECT uid FROM book where page_id=$1",
            &[&body.delete_node_id],
        )
        .await?;

    if delete_rows.len() > 0 {
        for row in delete_rows.iter() {
            let uid = row.get(0);
            delete_row_ids.push(uid);
        }
    }

    if *&body.identity == 101 {
        let delete_rows_two = conn
            .query(
                "SELECT uid FROM book where page_id=ANY($1)",
                &[&delete_row_ids],
            )
            .await?;

        if delete_rows_two.len() > 0 {
            for row2 in delete_rows_two.iter() {
                let uid2 = row2.get(0);
                delete_row_ids.push(uid2);
            }
        }
    }

    let u = conn
        .query_opt(
            "SELECT uid, parent_id from book where parent_id=$1 AND identity=$2",
            &[&body.delete_node_id, &body.identity],
        )
        .await?;

    let delete_nodes_query =
        delete_nodes_query!("book", "uid", &delete_row_ids, &body.delete_node_id);
    delete_row_ids.push(body.delete_node_id);

    let mut return_me = json!({
        "deleted_ids": delete_row_ids
    });
    if let Some(update_row) = u {
        let update_id: i32 = update_row.get(0);

        let update_node = update_query!(
            "book",
            "parent_id",
            &body.update_parent_id,
            "uid",
            update_id
        );
        let thisquery = format!("{}; {};", &delete_nodes_query, &update_node);
        println!("{}", thisquery);
        conn.batch_execute(&thisquery).await?;
        return_me = json!({
            "update_id": update_id,
            "deleted_ids": delete_row_ids,
        });
    } else {
        conn.query_one(&delete_nodes_query, &[&delete_row_ids])
            .await?;
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(return_me),
    ))
}

pub async fn get_all_books(State(pool): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query("SELECT book_id, title, body, images FROM books", &[])
        .await?;

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

#[derive(Deserialize, Serialize)]
pub struct BookNode {
    uid: i32,
    parent_id: Option<i32>,
    title: String,
    body: String,
    images: Option<String>,
    identity: i16,
    page_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct BookInfo {
    book_id: i32,
}

pub async fn get_all_book_nodes(
    State(pool): State<AppState>,
    query: Query<BookInfo>,
) -> Result<impl IntoResponse, AppError> {
    let book_info: BookInfo = query.0;

    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query(
            "SELECT uid, parent_id, title, body, images, identity, page_id FROM book where book_id=$1",
            &[&book_info.book_id],
        )
        .await?;

    let mut books: Vec<BookNode> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        let uid: i32 = rows[index].get(0);
        let parent_id: Option<i32> = rows[index].get(1);
        let title: String = rows[index].get(2);
        let body: String = rows[index].get(3);
        let images: Option<String> = rows[index].get(4);
        let identity: i16 = rows[index].get(5);
        let page_id: Option<i32> = rows[index].get(6);
        books.push(BookNode {
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
