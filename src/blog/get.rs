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

#[derive(Deserialize, Serialize)]
pub struct GetAllBlogs {
    uid: i32,
    title: String,
    body: String,
    images: String,
    created_at: DateTime<Utc>,
    doc_type: u8,
}

pub async fn get_all_blogs_by_page_no(
    State(pool): State<AppState>,
    AxumPath(page_no): AxumPath<i64>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let limit: i64 = 2;
    let offset: i64 = (page_no - 1) * limit;
    let rows = conn
        .query(
            "SELECT uid, title, body, images, created_at FROM blogs where deleted_at is null LIMIT $1 OFFSET $2",
            &[&limit, &offset],
        )
        .await?;

    let mut blogs: Vec<GetAllBlogs> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        blogs.push(GetAllBlogs {
            uid: rows[index].get(0),
            title: rows[index].get(1),
            body: rows[index].get(2),
            images: rows[index].get(3),
            created_at: rows[index].get(4),
            doc_type: 1,
        });
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "data": blogs
        })),
    ))
}

pub async fn get_all_blogs_by_user_id(
    State(pool): State<AppState>,
    AxumPath(user_id): AxumPath<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query(
            "SELECT uid, title, body, images, created_at FROM blogs where deleted_at is null and user_id=$1",
            &[&user_id],
        )
        .await?;

    let mut blogs: Vec<GetAllBlogs> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        blogs.push(GetAllBlogs {
            uid: rows[index].get(0),
            title: rows[index].get(1),
            body: rows[index].get(2),
            images: rows[index].get(3),
            created_at: rows[index].get(4),
            doc_type: 1,
        });
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "data": blogs
        })),
    ))
}

#[derive(Deserialize, Serialize)]
pub struct BlogNode {
    uid: i32,
    blog_id: i32,
    parent_id: Option<i32>,
    title: String,
    body: String,
    images: Option<String>,
    theme: i16,
}

#[derive(Deserialize)]
pub struct BlogNodesRequestById {
    blog_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct BlogInfo {
    uid: i32,
    user_id: i32,
    title: String,
    body: String,
    images: Option<String>,
    created_at: DateTime<Utc>,
}

pub async fn get_all_blog_nodes(
    State(pool): State<AppState>,
    query: Query<BlogNodesRequestById>,
) -> Result<impl IntoResponse, AppError> {
    let blog_request: BlogNodesRequestById = query.0;

    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query(
            "SELECT uid, parent_id, title, body, images, theme FROM blog where blog_id=$1 and deleted_at is null",
            &[&blog_request.blog_id],
        )
        .await?;
    let blog_row = conn
        .query_one(
            "SELECT uid, user_id, title, body, images, created_at FROM blogs where uid=$1",
            &[&blog_request.blog_id],
        )
        .await?;

    let blog_info = BlogInfo {
        uid: blog_row.get(0),
        user_id: blog_row.get(1),
        title: blog_row.get(2),
        body: blog_row.get(3),
        images: blog_row.get(4),
        created_at: blog_row.get(5),
    };

    let mut nodes: Vec<BlogNode> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        nodes.push(BlogNode {
            uid: rows[index].get(0),
            blog_id: blog_row.get(0),
            parent_id: rows[index].get(1),
            title: rows[index].get(2),
            body: rows[index].get(3),
            images: rows[index].get(4),
            theme: rows[index].get(5),
        });
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "nodes": nodes,
            "blog": blog_info
        })),
    ))
}

#[derive(Serialize)]
pub struct HomeBlogsResponse {
    uid: i32,
    title: String,
    body: String,
    images: String,
    created_at: DateTime<Utc>,
    doc_type: u8,
}

pub async fn get_all_blogs_liked_by_user(
    State(pool): State<AppState>,
    AxumPath(user_id): AxumPath<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let user_tags_query = "SELECT tag_id FROM user_tags where user_id=$1";
    let blog_ids_user_tags_query = "SELECT blog_id FROM blog_tags where tag_id=ANY($1)";
    let blog_ids_query = "SELECT uid, title, body, images, created_at FROM blogs where uid=ANY($1)";

    let mut tag_ids: Vec<i32> = Vec::new();
    let rows = conn.query(user_tags_query, &[&user_id]).await?;
    for row in rows.iter() {
        tag_ids.push(row.get(0));
    }

    let mut blog_ids: Vec<i32> = Vec::new();
    let rows = conn.query(blog_ids_user_tags_query, &[&tag_ids]).await?;
    for row in rows.iter() {
        blog_ids.push(row.get(0));
    }

    let mut blogs: Vec<HomeBlogsResponse> = Vec::new();
    let rows = conn.query(blog_ids_query, &[&blog_ids]).await?;
    for row in rows.iter() {
        blogs.push(HomeBlogsResponse {
            uid: row.get(0),
            title: row.get(1),
            body: row.get(2),
            images: row.get(3),
            created_at: row.get(4),
            doc_type: 1,
        });
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "data": blogs
        })),
    ))
}
