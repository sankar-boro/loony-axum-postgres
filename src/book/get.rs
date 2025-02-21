use std::collections::HashSet;

use crate::error::AppError;
use crate::{fetch_book_pages, AppState};
use axum::{
    extract::{Path as AxumPath, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::types::{Book, BookParentNode, NavNodes, ChildNode};
use crate::{fetch_books_by_user_id, fetch_books_by_book_ids};


pub async fn get_all_books_by_page_no(
    State(pool): State<AppState>,
    AxumPath(page_no): AxumPath<i64>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let limit: i64 = 2;
    let offset: i64 = (page_no - 1) * limit;
    let rows = conn
        .query(
            "SELECT uid, user_id, title, images, created_at FROM books where deleted_at is NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            &[&limit, &offset],
        )
        .await?;

    let mut books: Vec<Book> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        books.push(Book {
            uid:rows[index].get(0),
            user_id:rows[index].get(1),
            title: rows[index].get(2),
            images: rows[index].get(3),
            created_at: rows[index].get(4),
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
    let books = fetch_books_by_user_id!(&pool, user_id)?;
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(books),
    ))
}

#[derive(Deserialize)]
pub struct ChaptersByBookIdRequest {
    doc_id: i32,
}

#[derive(Deserialize)]
pub struct Chapter {
    doc_id: i32,
    page_id: i32
}

pub async fn get_chapter_details(
    State(pool): State<AppState>,
    query: Query<Chapter>,
) -> Result<impl IntoResponse, AppError> {
    let book_request: Chapter = query.0;

    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query(
            "SELECT uid, parent_id, title, content, images, identity, page_id FROM book where uid=$1 OR parent_id=$2 AND identity=103 AND deleted_at is null",
            &[&book_request.page_id, &book_request.page_id],
        )
        .await?;

    let nodes = rows
        .iter()
        .map(|row| ChildNode {
            uid: row.get(0),
            parent_id: row.get(1),
            title: row.get(2),
            content: row.get(3),
            images: row.get(4),
            identity: row.get(5),
            page_id: row.get(6),
        })
        .collect::<Vec<ChildNode>>();

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "nodes": nodes
        })),
    ))
}

#[derive(Deserialize)]
pub struct Section {
    doc_id: i32,
    page_id: i32
}

pub async fn get_section_details(
    State(pool): State<AppState>,
    query: Query<Section>,
) -> Result<impl IntoResponse, AppError> {
    let book_request: Section = query.0;

    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query(
            "SELECT uid, parent_id, title, content, images, identity, page_id FROM book where uid=$1 OR parent_id=$2 AND identity=103 AND deleted_at is null",
            &[&book_request.page_id, &book_request.page_id],
        )
        .await?;

    let nodes = rows
        .iter()
        .map(|row| ChildNode {
            uid: row.get(0),
            parent_id: row.get(1),
            title: row.get(2),
            content: row.get(3),
            images: row.get(4),
            identity: row.get(5),
            page_id: row.get(6),
        })
        .collect::<Vec<ChildNode>>();

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "nodes": nodes
        })),
    ))
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


#[derive(Deserialize, Serialize, Debug)]
pub struct BookChaptersAndSections {
    uid: i32,
    parent_id: Option<i32>,
    title: String,
    identity: i16,
    page_id: Option<i32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MainNode {
    uid: i32,
    parent_id: Option<i32>,
    title: String,
    content: String,
    images: Option<String>,
    identity: i16,
    page_id: Option<i32>,

}

pub async fn get_book_chapters_and_sections(
    State(pool): State<AppState>,
    query: Query<ChaptersByBookIdRequest>,
) -> Result<impl IntoResponse, AppError> {
    let doc_id: i32 = query.doc_id;
    let conn = pool.pg_pool.get().await?;
    let (books, main_node) = fetch_book_pages!(&conn, doc_id)?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "main_node": main_node,
            "child_nodes": books
        })),
    ))
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

// @End Get

pub async fn get_users_book(
    State(pool): State<AppState>,
    AxumPath(user_id): AxumPath<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;

    let doc_ids_user_tags_query = "SELECT doc_id FROM book_tags where user_id=$1";
    let mut seen: HashSet<i32> = HashSet::new();
    let mut doc_ids: Vec<i32> = Vec::new();
    let doc_id_rows = conn.query(doc_ids_user_tags_query, &[&user_id]).await?;

    if doc_id_rows.len() > 0 {
        for row in doc_id_rows.iter() {
            let doc_id: i32 = row.get(0);
            if seen.insert(doc_id) {
                doc_ids.push(doc_id);
            }
        }
    }

    let mut books: Vec<Book> = Vec::new();

    if doc_ids.len() > 0 {
        books = fetch_books_by_book_ids!(&conn, doc_ids)?;
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(books),
    ))
}
