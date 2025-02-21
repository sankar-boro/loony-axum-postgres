use std::collections::HashSet;

use crate::{error::AppError, fetch_book_nodes_by_page_id};
use crate::{fetch_book_pages, AppState};
use axum::{
    extract::{Path as AxumPath, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::types::{Book, BookNode, BookParentNode, NavNodes, ChildNode};
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

pub async fn get_book_chapters(
    State(pool): State<AppState>,
    query: Query<ChaptersByBookIdRequest>,
) -> Result<impl IntoResponse, AppError> {
    let book_request: ChaptersByBookIdRequest = query.0;

    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query(
            "SELECT uid, parent_id, title, content, images, identity, page_id FROM book where doc_id=$1 AND identity<=101 AND deleted_at is null and parent_id is not null",
            &[&book_request.doc_id],
        )
        .await?;
    let book_row = conn
        .query_one(
            "SELECT uid, user_id, title, content, images, created_at FROM books where uid=$1",
            &[&book_request.doc_id],
        )
        .await?;

    let main_node = BookParentNode {
        uid: book_row.get(0),
        doc_id: book_request.doc_id,
        user_id: book_row.get(1),
        title: book_row.get(2),
        content: book_row.get(3),
        images: book_row.get(4),
        created_at: book_row.get(5)
    };

    let mut child_nodes: Vec<BookNode> = Vec::new();
    for (index, _) in rows.iter().enumerate() {
        child_nodes.push(BookNode {
            uid: rows[index].get(0),
            doc_id: book_request.doc_id,
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
            "SELECT uid, parent_id, title, content, images, identity, page_id FROM book where doc_id=$1 AND uid=$2 OR parent_id=$3 AND deleted_at is null AND identity in(101, 103)",
            &[&book_request.doc_id, &book_request.page_id, &book_request.page_id],
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
            "SELECT uid, parent_id, title, content, images, identity, page_id FROM book where doc_id=$1 AND uid=$2 OR parent_id=$3 AND deleted_at is null AND identity in(102, 103)",
            &[&book_request.doc_id, &book_request.page_id, &book_request.page_id],
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
pub struct BookByPageId {
    doc_id: i32,
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
            "SELECT uid, parent_id, title, content, images, identity, page_id FROM book where doc_id=$1 AND page_id=$2 AND identity=102 and deleted_at is null",
            &[&book_info.doc_id, &book_info.page_id],
        )
        .await?;

    let mut books: Vec<BookNodesByPageId> = Vec::new();

    for (index, _) in rows.iter().enumerate() {

        books.push(BookNodesByPageId {
            uid: rows[index].get(0),
            parent_id: rows[index].get(1),
            title: rows[index].get(2),
            content: rows[index].get(3),
            images: rows[index].get(4),
            identity: rows[index].get(5),
            page_id: rows[index].get(6),
        })
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(books),
    ))
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

#[derive(Deserialize)]
pub struct BookBySectionId {
    doc_id: i32,
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
    let books = fetch_book_nodes_by_page_id!(&conn, &book_info.doc_id, &book_info.page_id)?;
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
