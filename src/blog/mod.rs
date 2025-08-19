pub mod delete;
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
    doc_id: i32,
    title: String,
    content: String,
    images: Vec<Images>,
}

#[derive(Deserialize, Serialize)]
pub struct GetBlog {
    doc_id: i32,
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
    let insert_blog_query = conn
        .prepare("INSERT INTO blog(user_id, doc_id, title, content, images) VALUES($1, $2, $3, $4, $5) RETURNING uid")
        .await?;

    let transaction = conn.transaction().await?;

    let row = transaction
        .query_one(
            &insert_blogs_query,
            &[&user_id, &body.title, &body.content, &images],
        )
        .await?;

    let doc_id: i32 = row.get(0);

    transaction
        .execute(
            &insert_blog_query,
            &[&user_id, &doc_id, &body.title, &body.content, &images],
        )
        .await?;

    let score: i32 = 1;

    transaction.commit().await?;

    // let mut all_tags: Vec<(i32, i32, &str, i32)> = Vec::new();
    // let tags = body.tags;
    // tags.iter().for_each(|x| {
    //     all_tags.push((doc_id, user_id, x, score));
    // });

    // conn.query(
    //     &insert_tags("blog_tags", "(doc_id, user_id, tag, score)", all_tags),
    //     &[],
    // )
    // .await?;

    let _ = &body.images.move_images(
        &pool.dirs.tmp_upload,
        &pool.dirs.blog_upload,
        user_id,
        doc_id,
    );

    let new_blog = json!({
        "doc_id": doc_id,
        "title": &body.title,
        "content": &body.content,
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

    // let doc_id: i32 = row.get(0);

    let state_2 = conn
        .prepare("UPDATE blog SET title=$1, content=$2, images=$3 WHERE uid=$4")
        .await?;
    let transaction = conn.transaction().await?;
    transaction
        .execute(
            &state_1,
            &[&body.title, &body.content, &images, &body.doc_id],
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
        body.doc_id,
    );
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(body),
    ))
}

#[derive(Deserialize, Serialize)]
pub struct AddBlogNode {
    doc_id: i32,
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
            "SELECT uid, parent_id from blog where parent_id=$1 and deleted_at is NULL",
            &[&body.parent_id],
        )
        .await;
    let images = &serde_json::to_string(&body.images).unwrap();

    let insert_statement = conn
        .prepare(
            "INSERT INTO blog(user_id, doc_id, parent_id, title, content, images) values($1, $2, $3, $4, $5, $6) returning uid"
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
                &body.doc_id,
                &body.parent_id,
                &body.title,
                &body.content,
                &images
            ],
        )
        .await?;

    let new_node_uid: i32 = new_node.get(0);
    
    // let mut update_row_uid: Option<i32> = None;
    let mut update_response: Option<UpdateNode> = None;
    if let Ok(update_row) = update_row {
        if !update_row.is_empty() {
            let update_row_uid: i32 = update_row.get(0);
            transaction
                .execute(&update_statement, &[&new_node_uid, &update_row_uid])
                .await?;
            update_response = Some(UpdateNode {
                uid: update_row_uid,
                parent_id: new_node_uid,
            })
        }
    }

    transaction.commit().await?;

    // if let Some(tags) = &body.tags {
    //     let score: i32 = 1;
    //     let mut all_tags: Vec<(i32, i32, &str, i32)> = Vec::new();
    //     tags.iter().for_each(|x| {
    //         all_tags.push((body.doc_id, user_id, x, score));
    //     });
    
    //     conn.query(
    //         &insert_tags("blog_tags", "(doc_id, user_id, tag, score)", all_tags),
    //         &[],
    //     )
    //     .await?;
    // }
    
    let _ = &body.images.move_images(
        &pool.dirs.tmp_upload,
        &pool.dirs.blog_upload,
        user_id,
        body.doc_id,
    );

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "new_node": {
                "uid": new_node_uid,
                "parent_id": &body.parent_id,
                "title": &body.title,
                "content": &body.content,
                "images": &images,
                "tags": &body.tags
            },
            "update_node": update_response
        })),
    ))
}


#[derive(Deserialize, Serialize)]
struct UpdateNode {
    uid: i32,
    parent_id: i32,
}


#[derive(Deserialize, Serialize)]
pub struct EditBlogNode {
    uid: i32,
    title: String,
    content: String,
    doc_id: i32,
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
        body.doc_id,
    );

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(body),
    ))
}
