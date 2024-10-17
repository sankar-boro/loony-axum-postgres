use crate::error::AppError;
use crate::AppState;
use axum::{
    extract::Path as AxumPath,
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct TagsResponse {
    uid: i32,
    name: String,
}
pub async fn get_all_tags_user_can_follow(
    State(pool): State<AppState>,
    AxumPath(user_id): AxumPath<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;

    let get_tags_res = conn
        .query(
            "SELECT t.uid, t.name
            FROM tags t
            LEFT JOIN user_tags ut ON t.uid = ut.tag_id AND ut.user_id = $1
            WHERE ut.tag_id IS NULL",
            &[&user_id],
        )
        .await?;

    let mut response = Vec::new();
    for row in get_tags_res {
        response.push(TagsResponse {
            uid: row.get(0),
            name: row.get(1),
        });
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(response),
    ))
}

pub async fn get_all_tags_user_has_followed(
    State(pool): State<AppState>,
    AxumPath(user_id): AxumPath<i32>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;

    let get_tags_res = conn
        .query(
            "SELECT t.uid, t.name
            FROM tags t
            LEFT JOIN user_tags ut ON t.uid = ut.tag_id AND ut.user_id = $1",
            &[&user_id],
        )
        .await?;

    let mut response = Vec::new();
    for row in get_tags_res {
        response.push(TagsResponse {
            uid: row.get(0),
            name: row.get(1),
        });
    }

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(response),
    ))
}

#[derive(Deserialize, Serialize)]
pub struct DeleteUserTag {
    tag_id: i32,
    user_id: i32,
}

type UserFollowedATag = DeleteUserTag;

pub async fn user_removed_a_followed_tag(
    State(pool): State<AppState>,
    Json(body): Json<DeleteUserTag>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;

    let _ = conn
        .query(
            "DELETE FROM user_tags where tag_id=$1 and user_id=$2",
            &[&body.tag_id, &body.user_id],
        )
        .await?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        "".to_owned(),
    ))
}

pub async fn user_followed_a_tag(
    State(pool): State<AppState>,
    Json(body): Json<UserFollowedATag>,
) -> Result<impl IntoResponse, AppError> {
    let conn = pool.pg_pool.get().await?;

    let _ = conn
        .query(
            "INSERT INTO user_tags (tag_id, user_id) VALUES($1, $2)",
            &[&body.tag_id, &body.user_id],
        )
        .await?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        "".to_owned(),
    ))
}
