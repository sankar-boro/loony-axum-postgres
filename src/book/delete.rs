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
pub struct DeleteBook {
    doc_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct DeleteBookNode {
    identity: i16,
    delete_id: i32,
    parent_id: i32,
}


#[derive(Deserialize, Serialize)]
struct UpdateNode {
    uid: i32,
    parent_id: i32,
}

pub async fn delete_book(
    State(pool): State<AppState>,
    Json(body): Json<DeleteBook>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = pool.pg_pool.conn.get().await?;
    let current_time = Local::now();

    let state1 = conn
        .prepare("UPDATE book SET deleted_at=$1 WHERE doc_id=$2")
        .await?;
    let state2 = conn
        .prepare("UPDATE books SET deleted_at=$1 WHERE uid=$2")
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
            "data": "book deleted"
        })),
    ))
}

pub async fn delete_book_node(
    State(pool): State<AppState>,
    Json(body): Json<DeleteBookNode>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = pool.pg_pool.conn.get().await?;

    // Prepare to find ids to delete
    // Applies only for nodes where identity is 101, 102
    let mut delete_row_ids: Vec<i32> = Vec::new();
    delete_row_ids.push(body.delete_id);

    let delete_rows = conn
        .query("SELECT uid FROM book where page_id=$1", &[&body.delete_id])
        .await?;

    if delete_rows.len() > 0 {
        for row in delete_rows.iter() {
            let uid = row.get(0);
            delete_row_ids.push(uid);
        }
    }

    if body.identity == 101 {
        let sub_section_nodes = conn
            .query(
                "SELECT uid FROM book where page_id=ANY($1)",
                &[&delete_row_ids],
            )
            .await?;

        if sub_section_nodes.len() > 0 {
            for sub_section in sub_section_nodes.iter() {
                let uid2 = sub_section.get(0);
                delete_row_ids.push(uid2);
            }
        }
    }
    // Check if there is a node to update
    let update_row_exist = conn
        .query_opt(
            "SELECT uid, parent_id from book where parent_id=$1 AND identity=$2 AND deleted_at IS NULL",
            &[&body.delete_id, &body.identity],
        )
        .await?;
    let current_time = Local::now();
    let state1 = conn
        .prepare("UPDATE book set deleted_at=$1 WHERE uid=ANY($2)")
        .await?;
    let update_bot_node_query = conn
        .prepare("UPDATE book SET parent_id=$1 WHERE uid=$2")
        .await?;
    let transaction = conn.transaction().await?;
    let num_deleted_rows = transaction
        .execute(&state1, &[&current_time, &delete_row_ids])
        .await?;
    
    let mut update_response: Option<UpdateNode> = None;
    if let Some(update_row) = update_row_exist {
        if !update_row.is_empty() {
            let update_id: i32 = update_row.get(0);
            transaction
            .execute(&update_bot_node_query, &[&body.parent_id, &update_id])
            .await?;
            update_response = Some(UpdateNode {
                uid: update_id,
                parent_id: body.parent_id,
            })
        }
    }
    transaction.commit().await?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "delete_nodes": delete_row_ids,
            "update_node": update_response,
            "rows": num_deleted_rows
        })),
    ))
}