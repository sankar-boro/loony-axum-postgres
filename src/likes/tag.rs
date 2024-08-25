use crate::error::AppError;
use crate::AppState;
use axum::{
    extract::Path as AxumPath,
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;

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
