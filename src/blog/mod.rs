use crate::error::AppError;
use crate::traits::{Images, MoveImages};
use crate::AppState;
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_sessions::Session;

#[derive(Deserialize, Serialize)]
pub struct CreateBlog {
    title: String,
    body: String,
    images: Vec<Images>,
    author_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct EditBlog {
    blog_id: i32,
    title: String,
    body: String,
    identity: i16,
}

#[derive(Deserialize, Serialize)]
pub struct GetBlog {
    blog_id: i32,
    title: String,
    body: String,
    images: String,
}

pub async fn create_blog(
    State(pool): State<AppState>,
    _: Session,
    Json(body): Json<CreateBlog>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let _ = &body
        .images
        .move_images(&pool.dirs.file_upload_tmp, &pool.dirs.file_upload);
    let images = &serde_json::to_string(&body.images).unwrap();
    let _ = &body
        .images
        .move_images(&pool.dirs.file_upload_tmp, &pool.dirs.file_upload);

    let row = conn
        .query_one(
            "INSERT INTO blogs(title, body, images, author_id) VALUES($1, $2, $3, $4) RETURNING blog_id",
            &[&body.title, &body.body, &images, &body.author_id],
        )
        .await
        ?;

    let blog_id: i32 = row.get(0);

    let _ = conn
        .query_one(
            "INSERT INTO blog(blog_id, title, body, images) VALUES($1, $2, $3, $4) RETURNING *",
            &[&blog_id, &body.title, &body.body, &images],
        )
        .await?;

    let new_blog = json!({
        "blog_id": blog_id,
        "title": &body.title.clone(),
        "body": &body.body.clone(),
        "images": &images,
        "author_id": &body.author_id,
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(new_blog),
    ))
}

pub async fn edit_blog(
    State(pool): State<AppState>,
    _: Session,
    Json(body): Json<EditBlog>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let row = conn
        .query_one(
            "UPDATE blogs SET title=$1 AND body=$2 WHERE blog_id=$3",
            &[&body.title, &body.body, &body.blog_id],
        )
        .await?;

    let blog_id: i32 = row.get(0);

    let _ = conn
        .query_one(
            "UPDATE blog SET title=$1 AND body=$2 WHERE blog_id=$3",
            &[&blog_id, &body.title, &body.body, &body.blog_id],
        )
        .await?;

    let edit_blog = json!({
        "blog_id": blog_id,
        "title": &body.title.clone(),
        "body": &body.body.clone(),
        "blog_id": &body.blog_id
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(edit_blog),
    ))
}

#[derive(Deserialize, Serialize)]
pub struct AddBlogNode {
    blog_id: i32,
    title: String,
    body: String,
    images: Vec<Images>,
    parent_id: i32,
}

pub async fn append_blog_node(
    State(pool): State<AppState>,
    _: Session,
    Json(body): Json<AddBlogNode>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;

    let update_row = conn
        .query_one(
            "SELECT uid, parent_id from blog where parent_id=$1",
            &[&body.parent_id],
        )
        .await;
    let images = &serde_json::to_string(&body.images).unwrap();

    let new_node = conn
    .query_one(
            "INSERT INTO blog(blog_id, parent_id, title, body, images) values($1, $2, $3, $4, $5) returning uid",
            &[&body.blog_id, &body.parent_id, &body.title, &body.body, &images],
        )
        .await
        ?;

    let new_node_uid: i32 = new_node.get(0);

    let mut update_row_uid: Option<i32> = None;
    if let Ok(update_row) = update_row {
        if !update_row.is_empty() {
            update_row_uid = update_row.get(0);

            conn.query_one(
                "UPDATE blog SET parent_id=$1 where uid=$2 RETURNING uid",
                &[&new_node_uid, &update_row_uid],
            )
            .await?;
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
                "images": &body.images
            },
            "update_node": {
                "update_row_id": update_row_uid,
                "update_row_parent_id": new_node_uid
            }
        })),
    ))
}

#[derive(Deserialize, Serialize)]
pub struct DeleteBlog {
    blog_id: i32,
}

pub async fn delete_blog(
    State(pool): State<AppState>,
    _: Session,
    Json(body): Json<DeleteBlog>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let _ = conn
        .execute("DELETE FROM blogs WHERE blog_id=$1", &[&body.blog_id])
        .await?;
    let _ = conn
        .execute("DELETE FROM blog WHERE blog_id=$1", &[&body.blog_id])
        .await?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "data": "blog deleted"
        })),
    ))
}

#[derive(Deserialize, Serialize)]
pub struct DeleteBlogNode {
    delete_node_id: i32,
    update_parent_id: i32,
    update_node_id: Option<i32>,
}

pub async fn delete_blog_node(
    State(pool): State<AppState>,
    Json(body): Json<DeleteBlogNode>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let _ = conn
        .execute("DELETE FROM blog WHERE uid=$1", &[&body.delete_node_id])
        .await?;

    let _ = conn
        .execute(
            "UPDATE blog set parent_id=$1 WHERE uid=$2",
            &[&body.update_parent_id, &body.update_node_id],
        )
        .await?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "data": "blog deleted"
        })),
    ))
}

pub async fn get_all_blogs(State(pool): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query("SELECT blog_id, title, body, images FROM blogs", &[])
        .await?;

    let mut blogs: Vec<GetBlog> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        let blog_id: i32 = rows[index].get(0);
        let title: String = rows[index].get(1);
        let body: String = rows[index].get(2);
        let images: String = rows[index].get(3);
        blogs.push(GetBlog {
            blog_id,
            title,
            body,
            images,
        })
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
}

#[derive(Deserialize)]
pub struct BlogInfo {
    blog_id: i32,
}

pub async fn get_all_blog_nodes(
    State(pool): State<AppState>,
    query: Query<BlogInfo>,
) -> Result<impl IntoResponse, AppError> {
    let blog_info: BlogInfo = query.0;

    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query(
            "SELECT uid, parent_id, title, body, images FROM blog where blog_id=$1",
            &[&blog_info.blog_id],
        )
        .await?;

    let mut blogs: Vec<BlogNode> = Vec::new();

    for (index, _) in rows.iter().enumerate() {
        let uid: i32 = rows[index].get(0);
        let parent_id: Option<i32> = rows[index].get(1);
        let title: String = rows[index].get(2);
        let body: String = rows[index].get(3);
        let images: Option<String> = rows[index].get(4);
        blogs.push(BlogNode {
            uid,
            parent_id,
            title,
            body,
            images,
        })
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
pub struct EditBlogNode {
    uid: i32,
    title: String,
    body: String,
    identity: i16,
    blog_id: i32,
}

pub async fn edit_blog_node(
    State(pool): State<AppState>,
    Json(body): Json<EditBlogNode>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;

    let _ = conn
        .execute(
            "UPDATE blog SET title=$1, body=$2 WHERE uid=$3",
            &[&body.title, &body.body, &body.uid],
        )
        .await?;

    if *&body.identity == 100 {
        let _ = conn
            .execute(
                "UPDATE blogs SET title=$1, body=$2 WHERE blog_id=$3",
                &[&body.title, &body.body, &body.blog_id],
            )
            .await?;
    }

    let edit_blog = json!({
        "status": 200,
        "message": "UPDATED",
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(edit_blog),
    ))
}
