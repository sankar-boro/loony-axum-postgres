use crate::{error::AppError, AppState};
use axum::{
    extract::{Multipart, Path as AxumPath, State},
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
) -> Result<impl IntoResponse, AppError> {
    if let Some(field) = multipart.next_field().await? {
        let _ = field.name().unwrap().to_string();
        let filename = field.file_name().unwrap().to_string();
        let _ = field.content_type().unwrap().to_string();
        let data = field.bytes().await?;
        let this_uuid = Uuid::new_v4().to_string();
        let extension = Path::new(&filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap();
        let new_filename = format!("{}.{}", &this_uuid, &extension);
        let uploaded_filename = format!("{}/{}", &state.dirs.file_upload_tmp, &new_filename);
        let upload_path = Path::new(&uploaded_filename);
        let mut image_file = File::create(&upload_path)?;
        image_file.write_all(&data)?;
        let new_upload = json!({
            "status": 200,
            "message": "Uploaded",
            "data": {
                "uploaded_filename": &new_filename
            }
        });

        return Ok((
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            Json(new_upload),
        ));
    }

    Ok((
        StatusCode::NOT_FOUND,
        [(header::CONTENT_TYPE, "application/json")],
        Json(json!({
            "status": StatusCode::NOT_FOUND.to_string(),
            "message": "Image not found",
        })),
    ))
}

pub async fn get_file(
    State(state): State<AppState>,
    AxumPath(filename): AxumPath<String>,
) -> Result<impl IntoResponse, AppError> {
    // Assuming files are stored in a directory named 'files'
    let file_path = format!("{}/{}", &state.dirs.file_upload, filename);

    // Attempt to read the file contents
    let f = std::fs::read(&file_path)?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        f,
    ))
}

pub async fn get_uploaded_file(
    State(state): State<AppState>,
    AxumPath(filename): AxumPath<String>,
) -> Result<impl IntoResponse, AppError> {
    // Assuming files are stored in a directory named 'files'
    let file_path = format!("{}/{}", &state.dirs.file_upload_tmp, filename);

    // Attempt to read the file contents
    let f = std::fs::read(&file_path)?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        f,
    ))
}
