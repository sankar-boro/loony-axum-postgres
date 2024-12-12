use std::collections::HashSet;

use crate::error::AppError;
use crate::AppState;
use axum::{
    extract::{Path as AxumPath, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Debug)]
pub struct HomeBooksResponse {
    uid: i32,
    title: String,
    content: String,
    images: String,
    created_at: DateTime<Utc>,
    doc_type: u8,
}

pub async fn get_all_books_by_page_no(
    State(pool): State<AppState>,
    AxumPath(page_no): AxumPath<i64>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let limit: i64 = 2;
    let offset: i64 = (page_no - 1) * limit;
    let rows = conn
        .query(
            "SELECT uid, title, content, images, created_at FROM books where deleted_at is NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            &[&limit, &offset],
        )
        .await?;

    let mut books: Vec<HomeBooksResponse> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        let uid: i32 = rows[index].get(0);
        let title: String = rows[index].get(1);
        let content: String = rows[index].get(2);
        let images: String = rows[index].get(3);
        let created_at: DateTime<Utc> = rows[index].get(4);
        books.push(HomeBooksResponse {
            uid,
            title,
            content,
            images,
            created_at,
            doc_type: 2,
        })
    }
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(books),
    ))
}

pub async fn get_all_books_by_user_id(
    State(pool): State<AppState>,
    AxumPath(user_id): AxumPath<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query(
            "SELECT uid, title, content, images, created_at FROM books where deleted_at is NULL and user_id=$1",
            &[&user_id],
        )
        .await?;

    let mut books: Vec<HomeBooksResponse> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        let uid: i32 = rows[index].get(0);
        let title: String = rows[index].get(1);
        let content: String = rows[index].get(2);
        let images: String = rows[index].get(3);
        let created_at: DateTime<Utc> = rows[index].get(4);
        books.push(HomeBooksResponse {
            uid,
            title,
            content,
            images,
            created_at,
            doc_type: 2,
        })
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(books),
    ))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ChaptersByBookId {
    uid: i32,
    parent_id: Option<i32>,
    title: String,
    content: String,
    images: Option<String>,
    identity: i16,
    page_id: Option<i32>,
}

#[derive(Deserialize, Serialize)]
pub struct BookInfo {
    book_id: i32,
    user_id: i32,
    title: String,
    content: String,
    images: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct ChaptersByBookIdRequest {
    book_id: i32,
}

pub async fn get_book_chapters(
    State(pool): State<AppState>,
    query: Query<ChaptersByBookIdRequest>,
) -> Result<impl IntoResponse, AppError> {
    let book_request: ChaptersByBookIdRequest = query.0;

    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query(
            "SELECT uid, parent_id, title, content, images, identity, page_id FROM book where book_id=$1 AND identity<=101 AND deleted_at is null",
            &[&book_request.book_id],
        )
        .await?;
    let book_row = conn
        .query_one(
            "SELECT uid, user_id, title, content, images, created_at FROM books where uid=$1",
            &[&book_request.book_id],
        )
        .await?;

    let main_node = BookInfo {
        book_id: book_row.get(0),
        user_id: book_row.get(1),
        title: book_row.get(2),
        content: book_row.get(3),
        images: book_row.get(4),
        created_at: book_row.get(5),
    };

    let mut child_nodes: Vec<ChaptersByBookId> = Vec::new();
    for (index, _) in rows.iter().enumerate() {
        child_nodes.push(ChaptersByBookId {
            uid: rows[index].get(0),
            parent_id: rows[index].get(1),
            title: rows[index].get(2),
            content: rows[index].get(3),
            images: rows[index].get(4),
            identity: rows[index].get(5),
            page_id: rows[index].get(6),
        });
    }
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "child_nodes": child_nodes,
            "main_node": main_node
        })),
    ))
}

#[derive(Deserialize)]
pub struct BookByPageId {
    book_id: i32,
    page_id: i32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BookNodesByPageId {
    uid: i32,
    parent_id: Option<i32>,
    title: String,
    content: String,
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
            "SELECT uid, parent_id, title, content, images, identity, page_id FROM book where book_id=$1 AND page_id=$2 AND identity=102 and deleted_at is null",
            &[&book_info.book_id, &book_info.page_id],
        )
        .await?;

    let mut books: Vec<BookNodesByPageId> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        let uid: i32 = rows[index].get(0);
        let parent_id: Option<i32> = rows[index].get(1);
        let title: String = rows[index].get(2);
        let content: String = rows[index].get(3);
        let images: Option<String> = rows[index].get(4);
        let identity: i16 = rows[index].get(5);
        let page_id: Option<i32> = rows[index].get(6);

        books.push(BookNodesByPageId {
            uid,
            parent_id,
            title,
            content,
            images,
            identity,
            page_id,
        })
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(books),
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
    content: String,
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
            "SELECT uid, parent_id, title, content, images, identity, page_id FROM book where book_id=$1 AND page_id=$2 AND identity=103 and deleted_at is null",
            &[&book_info.book_id, &book_info.page_id],
        )
        .await?;

    let mut books: Vec<BookNodesBySectionId> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        let uid: i32 = rows[index].get(0);
        let parent_id: Option<i32> = rows[index].get(1);
        let title: String = rows[index].get(2);
        let content: String = rows[index].get(3);
        let images: Option<String> = rows[index].get(4);
        let identity: i16 = rows[index].get(5);
        let page_id: Option<i32> = rows[index].get(6);

        books.push(BookNodesBySectionId {
            uid,
            parent_id,
            title,
            content,
            images,
            identity,
            page_id,
        })
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(books),
    ))
}

// @End Get

pub async fn get_users_book(
    State(pool): State<AppState>,
    AxumPath(user_id): AxumPath<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;

    let book_ids_user_tags_query = "SELECT book_id FROM book_tags where user_id=$1";
    let mut seen: HashSet<i32> = HashSet::new();
    let mut book_ids: Vec<i32> = Vec::new();
    let book_id_rows = conn.query(book_ids_user_tags_query, &[&user_id]).await?;

    if book_id_rows.len() > 0 {
        for row in book_id_rows.iter() {
            let book_id: i32 = row.get(0);
            if seen.insert(book_id) {
                book_ids.push(book_id);
            }
        }
    }

    let mut books: Vec<HomeBooksResponse> = Vec::new();

    if book_ids.len() > 0 {
        let books_query =
            "SELECT uid, title, content, images, created_at FROM books where uid=ANY($1)";
        let book_rows = conn.query(books_query, &[&book_ids]).await?;

        for row in book_rows.iter() {
            books.push(HomeBooksResponse {
                uid: row.get(0),
                title: row.get(1),
                content: row.get(2),
                images: row.get(3),
                created_at: row.get(4),
                doc_type: 2,
            });
        }
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(books),
    ))
}
