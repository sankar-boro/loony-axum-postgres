use crate::error::AppError;
use crate::AppState;
use axum::{
    extract::Path as AxumPath,
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;

pub async fn get_subscribed_users(
    axum::extract::Extension(crate::utils::UserId(user_id)): axum::extract::Extension<crate::utils::UserId>,
    State(pool): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.conn.get().await?;
    let rows = conn
        .query(
            "SELECT subscribed_id FROM subscription WHERE user_id=$1",
            &[&user_id],
        )
        .await?;
    let mut subscribed_ids: Vec<i32> = Vec::new();
    for row in rows.iter() {
        subscribed_ids.push(row.get(0));
    }
    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(subscribed_ids),
    ))
}

pub async fn subscribe_user(
    axum::extract::Extension(crate::utils::UserId(auth_user_id)): axum::extract::Extension<crate::utils::UserId>,
    State(pool): State<AppState>,
    AxumPath(user_id): AxumPath<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.conn.get().await?;
    let row = conn
        .query_one(
            "SELECT * FROM subscription where user_id=$1 and subscribed_id=$2",
            &[&auth_user_id, &user_id],
        )
        .await?;
    let message;
    if row.is_empty() {
        message = String::from("Created");
        conn.query_one(
            "INSERT INTO subscription(user_id, subscribed_id) VALUES($1, $2)",
            &[&auth_user_id, &user_id],
        )
        .await?;
    } else {
        message = String::from("Aldready subscribed.");
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({ "message" : message })),
    ))
}

pub async fn un_subscribe_user(
    axum::extract::Extension(crate::utils::UserId(auth_user_id)): axum::extract::Extension<crate::utils::UserId>,
    State(pool): State<AppState>,
    AxumPath(user_id): AxumPath<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.conn.get().await?;
    let row = conn
        .query_one(
            "SELECT * FROM subscription where user_id=$1 and subscribed_id=$2",
            &[&auth_user_id, &user_id],
        )
        .await?;

    let message;

    if row.is_empty() {
        message = String::from("Column does not exist.");
    } else {
        conn.query_one(
            "DELETE FROM subscription where user_id = $1 AND subscribed_id = $2",
            &[&auth_user_id, &user_id],
        )
        .await?;
        message = String::from("User unfollowed.");
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({ "message" : message })),
    ))
}
