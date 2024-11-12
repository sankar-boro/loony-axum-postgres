use crate::error::AppError;
use crate::AppState;
use axum::{
    extract::{Path as AxumPath, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AddDocRequest {
    name: String,
    username: String,
    password: String,
    url: String,
    metadata: String,
    user_id: u32,
}

pub async fn add(
    State(state): State<AppState>,
    Json(body): Json<AddDocRequest>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.pg_pool.get().await?;

    conn.execute(
        "INSERT INTO credentials(name, username, password, url, user_id, metadata) values($1, $2, $3, $4, $5, $6) RETURNING uid",
        &[&body.name, &body.username, &body.password, &body.url, &body.user_id, &body.metadata],
    )
    .await?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(body),
    ))
}

pub async fn delete(
    State(state): State<AppState>,
    AxumPath(delete_uid): AxumPath<i64>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.pg_pool.get().await?;

    conn.execute("DELETE from credentials WHERE uid=$1", &[&delete_uid])
        .await?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(serde_json::json!({ "status": 200 })),
    ))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EditDocRequest {
    name: String,
    username: String,
    password: String,
    url: String,
    metadata: String,
}

pub async fn edit(
    State(state): State<AppState>,
    Json(body): Json<EditDocRequest>,
) -> Result<impl IntoResponse, AppError> {
    let conn = state.pg_pool.get().await?;

    conn.execute("UPDATE credentials SET name=$1, username=$2, password=$3, url=$4, metadata=$5 WHERE uid=$6", &[&body.name, &body.username, &body.password, &body.url, &body.metadata])
        .await?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(serde_json::json!({ "status": 200 })),
    ))
}
