use crate::error::AppError;
use crate::AppState;
use axum::{
    extract::{Query, State, Path as AxumPath},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono::{DateTime, Utc};

#[derive(Deserialize, Serialize)]
pub struct GetAllBlogs {
    blog_id: i32,
    title: String,
    body: String,
    images: String,
    created_at: DateTime<Utc>,
}
pub async fn get_all_blogs(State(pool): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query(
            "SELECT blog_id, title, body, images, created_at FROM blogs where deleted_at is null",
            &[],
        )
        .await?;

    let mut blogs: Vec<GetAllBlogs> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        blogs.push(GetAllBlogs {
            blog_id:rows[index].get(0),
            title:rows[index].get(1),
            body:rows[index].get(2),
            images:rows[index].get(3),
            created_at:rows[index].get(4)
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
    AxumPath(user_id): AxumPath<i32>
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query(
            "SELECT blog_id, title, body, images, created_at FROM blogs where deleted_at is null and user_id=$1",
            &[&user_id],
        )
        .await?;

    let mut blogs: Vec<GetAllBlogs> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        blogs.push(GetAllBlogs {
            blog_id:rows[index].get(0),
            title:rows[index].get(1),
            body:rows[index].get(2),
            images:rows[index].get(3),
            created_at:rows[index].get(4)
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
    parent_id: Option<i32>,
    title: String,
    body: String,
    images: Option<String>,
    theme: i16
}

#[derive(Deserialize)]
pub struct BlogNodesRequestById {
    blog_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct BlogInfo {
    blog_id: i32,
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
            "SELECT blog_id, user_id, title, body, images, created_at FROM blogs where blog_id=$1",
            &[&blog_request.blog_id],
        )
        .await?;

    let blog_info = BlogInfo {
        blog_id: blog_row.get(0),
        user_id: blog_row.get(1),
        title:blog_row.get(2),
        body:blog_row.get(3),
        images: blog_row.get(4),
        created_at: blog_row.get(5)
    };

    let mut nodes: Vec<BlogNode> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        nodes.push(BlogNode {
            uid:rows[index].get(0),
            parent_id:rows[index].get(1),
            title:rows[index].get(2),
            body:rows[index].get(3),
            images:rows[index].get(4),
            theme:rows[index].get(5)
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