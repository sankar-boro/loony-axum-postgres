use crate::error::AppError;
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
pub struct DeleteBlog {
    doc_id: i32,
}

pub async fn delete_blog(
    State(pool): State<AppState>,
    Json(body): Json<DeleteBlog>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = pool.pg_pool.get().await?;
    let current_time = Local::now();
    let state1 = conn
        .prepare("UPDATE blogs SET deleted_at=$1 WHERE uid=$2")
        .await?;
    let state2 = conn
        .prepare("UPDATE blog SET deleted_at=$1 WHERE doc_id=$2")
        .await?;
    let transaction = conn.transaction().await?;
    transaction
        .execute(&state1, &[&current_time, &body.doc_id])
        .await?;
    transaction
        .execute(&state2, &[&current_time, &body.doc_id])
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