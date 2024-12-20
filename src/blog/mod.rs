pub mod get;
mod utils;

use crate::error::AppError;
use crate::traits::{Images, MoveImages};
use crate::utils::doc::insert_tags;
use crate::utils::GetUserId;
use crate::AppState;
use axum::{
    extract::State,
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
    content: String,
    images: Vec<Images>,
    tags: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct EditBlog {
    uid: i32,
    blog_id: i32,
    title: String,
    content: String,
    images: Vec<Images>,
}

#[derive(Deserialize, Serialize)]
pub struct GetBlog {
    blog_id: i32,
    title: String,
    content: String,
    images: String,
}

pub async fn create_blog(
    session: Session,
    State(pool): State<AppState>,
    Json(body): Json<CreateBlog>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = session.get_user_id().await?;
    let images = &serde_json::to_string(&body.images).unwrap();

    let mut conn = pool.pg_pool.get().await?;

    let insert_blogs_query = conn
        .prepare("INSERT INTO blogs(user_id, title, content, images) VALUES($1, $2, $3, $4) RETURNING uid")
        .await?;
    // let insert_blog_query = conn
    //     .prepare("INSERT INTO blog(user_id, blog_id, title, content, images) VALUES($1, $2, $3, $4, $5) RETURNING uid")
    //     .await?;

    let transaction = conn.transaction().await?;

    let row = transaction
        .query_one(
            &insert_blogs_query,
            &[&user_id, &body.title, &body.content, &images],
        )
        .await?;

    let blog_id: i32 = row.get(0);

    // transaction
    //     .execute(
    //         &insert_blog_query,
    //         &[&user_id, &blog_id, &body.title, &body.content, &images],
    //     )
    //     .await?;

    let score: i32 = 1;

    transaction.commit().await?;

    let mut all_tags: Vec<(i32, i32, &str, i32)> = Vec::new();
    let tags = body.tags;
    tags.iter().for_each(|x| {
        all_tags.push((blog_id, user_id, x, score));
    });

    conn.query(
        &insert_tags("blog_tags", "(blog_id, user_id, tag, score)", all_tags),
        &[],
    )
    .await?;

    let _ = &body.images.move_images(
        &pool.dirs.tmp_upload,
        &pool.dirs.blog_upload,
        user_id,
        blog_id,
    );

    let new_blog = json!({
        "blog_id": blog_id,
        "title": &body.title,
        "body": &body.content,
        "images": &images,
        "user_id": &user_id
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(new_blog),
    ))
}

pub async fn edit_blog(
    session: Session,
    State(pool): State<AppState>,
    Json(body): Json<EditBlog>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = session.get_user_id().await?;

    let mut conn = pool.pg_pool.get().await?;
    let images = &serde_json::to_string(&body.images).unwrap();

    let state_1 = conn
        .prepare("UPDATE blogs SET title=$1, content=$2, images=$3 WHERE uid=$4")
        .await?;

    // let blog_id: i32 = row.get(0);

    let state_2 = conn
        .prepare("UPDATE blog SET title=$1, content=$2, images=$3 WHERE uid=$4")
        .await?;
    let transaction = conn.transaction().await?;
    transaction
        .execute(
            &state_1,
            &[&body.title, &body.content, &images, &body.blog_id],
        )
        .await?;
    transaction
        .execute(&state_2, &[&body.title, &body.content, &images, &body.uid])
        .await?;
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
        Json(body),
    ))
}

#[derive(Deserialize, Serialize)]
pub struct AddBlogNode {
    blog_id: i32,
    title: String,
    content: String,
    images: Vec<Images>,
    parent_id: i32,
    tags: Option<Vec<String>>,
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

    let insert_statement = conn
        .prepare(
            "INSERT INTO blog(user_id, blog_id, parent_id, title, content, images) values($1, $2, $3, $4, $5, $6) returning uid"
        )
        .await?;

    let update_statement = conn
        .prepare("UPDATE blog SET parent_id=$1 where uid=$2 RETURNING uid")
        .await?;

    let transaction = conn.transaction().await?;
    let new_node = transaction
        .query_one(
            &insert_statement,
            &[
                &user_id,
                &body.blog_id,
                &body.parent_id,
                &body.title,
                &body.content,
                &images
            ],
        )
        .await?;

    let new_node_uid: i32 = new_node.get(0);
    
    let mut update_row_uid: Option<i32> = None;
    if let Ok(update_row) = update_row {
        if !update_row.is_empty() {
            update_row_uid = update_row.get(0);
            transaction
                .execute(&update_statement, &[&new_node_uid, &update_row_uid])
                .await?;
        }
    }

    transaction.commit().await?;

    if let Some(tags) = &body.tags {
        let score: i32 = 1;
        let mut all_tags: Vec<(i32, i32, &str, i32)> = Vec::new();
        tags.iter().for_each(|x| {
            all_tags.push((body.blog_id, user_id, x, score));
        });
    
        conn.query(
            &insert_tags("blog_tags", "(blog_id, user_id, tag, score)", all_tags),
            &[],
        )
        .await?;
    }
    
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
                "body": &body.content,
                "images": &images,
                "tags": &body.tags
            },
            "update_node": {
                "uid": update_row_uid,
                "parent_id": new_node_uid
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
struct DeleteNode {
    uid: i32,
}
#[derive(Deserialize, Serialize)]
struct UpdateNode {
    uid: i32,
    parent_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct DeleteBlogNode {
    delete_node: DeleteNode,
    update_node: Option<UpdateNode>,
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
        .execute(&state1, &[&current_time, &body.delete_node.uid])
        .await?;
    if let Some(update_node) = &body.update_node {
        transaction
            .execute(
                &state2,
                &[&update_node.parent_id, &update_node.uid],
            )
            .await?;
    }
    transaction.commit().await?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(body),
    ))
}

#[derive(Deserialize, Serialize)]
pub struct EditBlogNode {
    uid: i32,
    title: String,
    content: String,
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
        .prepare("UPDATE blog SET title=$1, content=$2, images=$3 WHERE uid=$4")
        .await?;
    let transaction = conn.transaction().await?;

    transaction
        .execute(&state1, &[&body.title, &body.content, &images, &body.uid])
        .await?;
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
        Json(body),
    ))
}
