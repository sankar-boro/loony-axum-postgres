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
use serde_json::json;
use tower_sessions::Session;
use chrono::{DateTime, Utc};

#[derive(Deserialize, Serialize)]
pub struct CreateBlog {
    title: String,
    body: String,
    images: Vec<Images>,
    tags: Option<String>,
    theme: i16,
}

#[derive(Deserialize, Serialize)]
pub struct EditBlog {
    blog_id: i32,
    title: String,
    body: String,
    images: Vec<Images>,
    theme: i16
}

#[derive(Deserialize, Serialize)]
pub struct GetBlog {
    blog_id: i32,
    title: String,
    body: String,
    images: String,
    theme: i16
}

pub async fn create_blog(
    session: Session,
    State(pool): State<AppState>,
    Json(body): Json<CreateBlog>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = session.get_user_id().await?;
    let mut conn = pool.pg_pool.get().await?;
    let images = &serde_json::to_string(&body.images).unwrap();

    let state1 = conn
        .prepare(
            "INSERT INTO blogs(title, body, images, user_id, tags, theme) VALUES($1, $2, $3, $4, $5, $6) RETURNING blog_id"
        )
        .await?;
    let state2 = conn
        .prepare(
            "INSERT INTO blog(blog_id, title, body, images, tags, theme) VALUES($1, $2, $3, $4, $5, $6) RETURNING *",
        )
        .await?;

    let transaction = conn.transaction().await?;
    let row = transaction
        .query_one(
            &state1,
            &[&body.title, &body.body, &images, &user_id, &body.tags, &body.theme],
        )
        .await?;
    let blog_id: i32 = row.get(0);
    transaction
        .execute(
            &state2,
            &[&blog_id, &body.title, &body.body, &images, &body.tags, &body.theme],
        )
        .await?;
    transaction.commit().await?;

    let _ = &body.images.move_images(
        &pool.dirs.tmp_upload,
        &pool.dirs.blog_upload,
        user_id,
        blog_id,
    );

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "blog_id": blog_id,
            "title": &body.title.clone(),
            "body": &body.body.clone(),
            "images": &images,
            "user_id": &user_id,
            "tags": &body.tags,
            "theme": &body.theme
        })),
    ))
}

pub async fn edit_blog(
    session: Session,
    State(pool): State<AppState>,
    Json(body): Json<EditBlog>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = session.get_user_id().await?;

    let conn = pool.pg_pool.get().await?;
    let images = &serde_json::to_string(&body.images).unwrap();

    let row = conn
        .query_one(
            "UPDATE blogs SET title=$1, body=$2, images=$3, theme=$4 WHERE blog_id=$5",
            &[&body.title, &body.body, &images, &body.theme, &body.blog_id],
        )
        .await?;

    let blog_id: i32 = row.get(0);

    let _ = conn
        .query_one(
            "UPDATE blog SET title=$1, body=$2, images=$3, theme=$4 WHERE blog_id=$5",
            &[&blog_id, &body.title, &body.body, &images, &body.theme, &body.blog_id],
        )
        .await?;

    let edit_blog = json!({
        "blog_id": blog_id,
        "title": &body.title.clone(),
        "body": &body.body.clone(),
        "images": &images,
        "blog_id": &body.blog_id,
        "theme": &body.theme
    });
    let _ = &body.images.move_images(
        &pool.dirs.tmp_upload,
        &pool.dirs.blog_upload,
        user_id,
        blog_id,
    );
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
    tags: Option<String>,
    theme: i16
}

pub async fn append_blog_node(
    session: Session,
    State(pool): State<AppState>,
    Json(body): Json<AddBlogNode>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = session.get_user_id().await?;
    let mut conn = pool.pg_pool.get().await?;

    let update_row = conn
        .query_one(
            "SELECT uid, parent_id from blog where parent_id=$1",
            &[&body.parent_id],
        )
        .await;
    let images = &serde_json::to_string(&body.images).unwrap();

    let state1 = conn
        .prepare(
            "INSERT INTO blog(blog_id, parent_id, title, body, images, tags, theme) values($1, $2, $3, $4, $5, $6, $7) returning uid"
        )
        .await?;

    let state2 = conn
        .prepare("UPDATE blog SET parent_id=$1 where uid=$2 RETURNING uid")
        .await?;

    let transaction = conn.transaction().await?;
    let new_node = transaction
        .query_one(
            &state1,
            &[
                &body.blog_id,
                &body.parent_id,
                &body.title,
                &body.body,
                &images,
                &body.tags,
                &body.theme
            ],
        )
        .await?;

    let new_node_uid: i32 = new_node.get(0);

    let mut update_row_uid: Option<i32> = None;
    if let Ok(update_row) = update_row {
        if !update_row.is_empty() {
            update_row_uid = update_row.get(0);
            transaction
                .execute(&state2, &[&new_node_uid, &update_row_uid])
                .await?;
        }
    }
    transaction.commit().await?;

    let _ = &body.images.move_images(
        &pool.dirs.tmp_upload,
        &pool.dirs.blog_upload,
        user_id,
        body.blog_id,
    );

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
                "tags": &body.tags,
                "theme": &body.theme
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
    Json(body): Json<DeleteBlog>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = pool.pg_pool.get().await?;
    let current_time = Local::now();
    let state1 = conn
        .prepare("UPDATE blogs SET deleted_at=$1 WHERE blog_id=$2")
        .await?;
    let state2 = conn
        .prepare("UPDATE blog SET deleted_at=$1 WHERE blog_id=$2")
        .await?;
    let transaction = conn.transaction().await?;
    transaction
        .execute(&state1, &[&current_time, &body.blog_id])
        .await?;
    transaction
        .execute(&state2, &[&current_time, &body.blog_id])
        .await?;
    transaction.commit().await?;
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
    let mut conn = pool.pg_pool.get().await?;
    let current_time = Local::now();

    let state1 = conn
        .prepare("UPDATE blog set deleted_at=$1 WHERE uid=$2")
        .await?;

    let state2 = conn
        .prepare("UPDATE blog set parent_id=$1 WHERE uid=$2")
        .await?;
    let transaction = conn.transaction().await?;
    transaction
        .execute(&state1, &[&current_time, &body.delete_node_id])
        .await?;
    transaction
        .execute(&state2, &[&body.update_parent_id, &body.update_node_id])
        .await?;
    transaction.commit().await?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "data": "blog deleted"
        })),
    ))
}

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

#[derive(Deserialize, Serialize)]
pub struct EditBlogNode {
    uid: i32,
    title: String,
    body: String,
    blog_id: i32,
    images: Vec<Images>,
    theme: i16
}

pub async fn edit_blog_node(
    session: Session,
    State(pool): State<AppState>,
    Json(body): Json<EditBlogNode>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = session.get_user_id().await?;
    let mut conn = pool.pg_pool.get().await?;
    let images = &serde_json::to_string(&body.images).unwrap();
    let state1 = conn
        .prepare("UPDATE blog SET title=$1, body=$2, images=$3, theme=$4 WHERE uid=$5")
        .await?;
    let transaction = conn.transaction().await?;

    transaction
        .execute(&state1, &[&body.title, &body.body, &images, &body.theme, &body.uid])
        .await?;
    transaction.commit().await?;
    let _ = &body.images.move_images(
        &pool.dirs.tmp_upload,
        &pool.dirs.blog_upload,
        user_id,
        body.blog_id,
    );
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
