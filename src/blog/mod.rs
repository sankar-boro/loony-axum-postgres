pub mod get;

use crate::error::AppError;
use crate::traits::{Images, MoveImages};
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
    body: String,
    images: Vec<Images>,
    tags: Option<Vec<String>>,
    theme: i16,
}

#[derive(Deserialize, Serialize)]
pub struct EditBlog {
    uid: i32,
    blog_id: i32,
    title: String,
    body: String,
    images: Vec<Images>,
    theme: i16,
}

#[derive(Deserialize, Serialize)]
pub struct GetBlog {
    blog_id: i32,
    title: String,
    body: String,
    images: String,
    theme: i16,
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
            "INSERT INTO blogs(title, body, images, user_id, theme) VALUES($1, $2, $3, $4, $5) RETURNING uid"
        )
        .await?;
    let state2 = conn
        .prepare(
            "INSERT INTO blog(uid, title, body, images, theme) VALUES($1, $2, $3, $4, $5) RETURNING *",
        )
        .await?;

    let mut insert_tags_query: Option<String> = None;
    if let Some(tags) = &body.tags {
        insert_tags_query = Some(format!(
            "WITH ins AS (
                INSERT INTO tags (name)
                VALUES {}
                ON CONFLICT (name) DO NOTHING
                RETURNING uid, name
            )
            SELECT uid, name FROM ins
            UNION ALL
            SELECT uid, name FROM tags WHERE name IN ({}) AND NOT EXISTS (SELECT uid, name FROM ins)",
            tags.iter()
                .map(|s| format!("('{}')", s))
                .collect::<Vec<String>>()
                .join(", "),
                tags.iter()
                .map(|s| format!("'{}'", s))
                .collect::<Vec<String>>()
                .join(", ")
        ));
    }

    let transaction = conn.transaction().await?;
    let row = transaction
        .query_one(
            &state1,
            &[&body.title, &body.body, &images, &user_id, &body.theme],
        )
        .await?;
    let blog_id: i32 = row.get(0);

    if let Some(insert_tags_query) = insert_tags_query {
        let res = transaction.query(&insert_tags_query, &[]).await?;
        let mut tag_rows: Vec<(i32, i32, i32)> = Vec::new();
        for row in res.iter() {
            tag_rows.push((row.get(0), blog_id, user_id));
        }
        let blog_tag_query = format!(
            "INSERT INTO blog_tags(tag_id, blog_id) VALUES {} RETURNING uid",
            tag_rows
                .iter()
                .map(|(tid, bid, _)| format!("('{}', '{}')", tid, bid))
                .collect::<Vec<String>>()
                .join(", "),
        );
        let user_tag_query = format!(
            "INSERT INTO user_tags(tag_id, user_id) VALUES {} RETURNING uid",
            tag_rows
                .iter()
                .map(|(tid, _, uid)| format!("('{}', '{}')", tid, uid))
                .collect::<Vec<String>>()
                .join(", "),
        );
        transaction.execute(&blog_tag_query, &[]).await?;
        transaction.execute(&user_tag_query, &[]).await?;
    }

    transaction
        .execute(
            &state2,
            &[&blog_id, &body.title, &body.body, &images, &body.theme],
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
            "uid": blog_id,
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

    let mut conn = pool.pg_pool.get().await?;
    let images = &serde_json::to_string(&body.images).unwrap();

    let state_1 = conn
        .prepare("UPDATE blogs SET title=$1, body=$2, images=$3, theme=$4 WHERE uid=$5")
        .await?;

    // let blog_id: i32 = row.get(0);

    let state_2 = conn
        .prepare("UPDATE blog SET title=$1, body=$2, images=$3, theme=$4 WHERE uid=$5")
        .await?;
    let transaction = conn.transaction().await?;
    transaction
        .execute(
            &state_1,
            &[&body.title, &body.body, &images, &body.theme, &body.blog_id],
        )
        .await?;
    transaction
        .execute(
            &state_2,
            &[&body.title, &body.body, &images, &body.theme, &body.uid],
        )
        .await?;

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
    body: String,
    images: Vec<Images>,
    parent_id: i32,
    tags: Option<String>,
    theme: i16,
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
            "INSERT INTO blog(blog_id, parent_id, title, body, images, tags, theme) values($1, $2, $3, $4, $5, $6, $7) returning uid"
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
                &body.blog_id,
                &body.parent_id,
                &body.title,
                &body.body,
                &images,
                &body.tags,
                &body.theme,
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
    update_node: UpdateNode,
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
    transaction
        .execute(
            &state2,
            &[&body.update_node.parent_id, &body.update_node.uid],
        )
        .await?;
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
    body: String,
    blog_id: i32,
    images: Vec<Images>,
    theme: i16,
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
        .execute(
            &state1,
            &[&body.title, &body.body, &images, &body.theme, &body.uid],
        )
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
