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

#[derive(Deserialize, Serialize)]
pub struct CreateBlog {
    title: String,
    body: String,
    images: Vec<Images>,
    tags: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct EditBlog {
    blog_id: i32,
    title: String,
    body: String,
    identity: i16,
    images: Vec<Images>,
}

#[derive(Deserialize, Serialize)]
pub struct GetBlog {
    blog_id: i32,
    title: String,
    body: String,
    images: String,
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
            "INSERT INTO blogs(title, body, images, user_id, tags) VALUES($1, $2, $3, $4, $5) RETURNING blog_id"
        )
        .await?;
    let state2 = conn
        .prepare(
            "INSERT INTO blog(blog_id, title, body, images, tags) VALUES($1, $2, $3, $4, $5) RETURNING *",
        )
        .await?;

    let transaction = conn.transaction().await?;
    let row = transaction
        .query_one(
            &state1,
            &[&body.title, &body.body, &images, &user_id, &body.tags],
        )
        .await?;
    let blog_id: i32 = row.get(0);
    transaction
        .execute(
            &state2,
            &[&blog_id, &body.title, &body.body, &images, &body.tags],
        )
        .await?;
    transaction.commit().await?;

    let _ = &body.images.move_images(
        &pool.dirs.file_upload_tmp,
        &pool.dirs.file_upload_doc,
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
            "tags": &body.tags
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
            "UPDATE blogs SET title=$1, body=$2, images=$3 WHERE blog_id=$4",
            &[&body.title, &body.body, &images, &body.blog_id],
        )
        .await?;

    let blog_id: i32 = row.get(0);

    let _ = conn
        .query_one(
            "UPDATE blog SET title=$1, body=$2, images=$3 WHERE blog_id=$4",
            &[&blog_id, &body.title, &body.body, &images, &body.blog_id],
        )
        .await?;

    let edit_blog = json!({
        "blog_id": blog_id,
        "title": &body.title.clone(),
        "body": &body.body.clone(),
        "images": &images,
        "blog_id": &body.blog_id
    });
    let _ = &body.images.move_images(
        &pool.dirs.file_upload_tmp,
        &pool.dirs.file_upload_doc,
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
}

pub async fn append_blog_node(
    session: Session,
    State(pool): State<AppState>,
    Json(body): Json<AddBlogNode>,
) -> Result<impl IntoResponse, AppError> {
    let user_id: i32 = match session.get("AUTH_USER_ID").await {
        Ok(x) => match x {
            Some(x) => x,
            None => {
                return Err(AppError::InternalServerError(
                    "User session not found".to_string(),
                ))
            }
        },
        Err(e) => return Err(AppError::InternalServerError(e.to_string())),
    };
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
            "INSERT INTO blog(blog_id, parent_id, title, body, images, tags) values($1, $2, $3, $4, $5, $6) returning uid"
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
        &pool.dirs.file_upload_tmp,
        &pool.dirs.file_upload_doc,
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
                "images": &body.images,
                "tags": &body.tags
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

pub async fn get_all_blogs(State(pool): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;
    let rows = conn
        .query(
            "SELECT blog_id, title, body, images FROM blogs where deleted_at is null",
            &[],
        )
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
            "SELECT uid, parent_id, title, body, images FROM blog where blog_id=$1 and deleted_at is null",
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
    images: Vec<Images>,
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
        .prepare("UPDATE blog SET title=$1, body=$2, images=$3 WHERE uid=$4")
        .await?;
    let state2 = conn
        .prepare("UPDATE blogs SET title=$1, body=$2, images=$3 WHERE blog_id=$4")
        .await?;
    let transaction = conn.transaction().await?;

    transaction
        .execute(&state1, &[&body.title, &body.body, &images, &body.uid])
        .await?;
    if *&body.identity == 100 {
        transaction
            .execute(&state2, &[&body.title, &body.body, &images, &body.blog_id])
            .await?;
    }
    transaction.commit().await?;
    let _ = &body.images.move_images(
        &pool.dirs.file_upload_tmp,
        &pool.dirs.file_upload_doc,
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
