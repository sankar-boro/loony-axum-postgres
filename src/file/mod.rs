use crate::{error::internal_error, AppState};
use axum::{
    extract::{Multipart, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::{fs::File, io::Write, path::Path};
use uuid::Uuid;

// Doesn't support image upload size greater then 2mb
pub async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if let Some(field) = multipart.next_field().await.map_err(internal_error)? {
        let _ = field.name().unwrap().to_string();
        let filename = field.file_name().unwrap().to_string();
        let _ = field.content_type().unwrap().to_string();
        let data = field.bytes().await.map_err(internal_error)?;
        // let local = OffsetDateTime::now_local().unwrap();
        let this_uuid = Uuid::new_v4().to_string();
        let extension = Path::new(&filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap();
        let pp = format!(
            "{}/{}.{}",
            &state.dirs.file_upload_tmp, &this_uuid, &extension
        );
        let upload_path = Path::new(&pp);
        let mut image_file = File::create(&upload_path).map_err(internal_error)?;
        image_file.write_all(&data).map_err(internal_error)?;
    }

    let new_book = json!({
        "status": 200,
        "message": "Uploaded"
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(new_book),
    ))
}
