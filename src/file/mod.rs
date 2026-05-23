use crate::types::ImageMetadata;
use crate::utils::{new_height, GetUserId};
use crate::{error::AppError, AppState};
use axum::{
    extract::{Multipart, Path as AxumPath, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use image::{imageops::FilterType, ImageFormat};
use serde_json::json;
use std::io::Cursor;
use std::path::Path;
use tower_sessions::Session;
use uuid::Uuid;

const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif"];

fn is_valid_image(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\xFF\xD8\xFF")          // JPEG
    || bytes.starts_with(b"\x89PNG\r\n\x1a\n")  // PNG
    || bytes.starts_with(b"GIF87a")             // GIF87a
    || bytes.starts_with(b"GIF89a")             // GIF89a
    || (bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP")) // WebP
}

fn validate_path_component(s: &str) -> Result<(), AppError> {
    if s.contains("..") || s.contains('/') || s.contains('\\') || s.contains('\0') {
        return Err(AppError::BadRequest("Invalid path component".into()));
    }
    Ok(())
}

async fn create_tmp_path(
    state: &AppState,
    session: &Session,
    extension: &str,
) -> Result<(String, String, String, String, u32), AppError> {
    let unique_uuid = Uuid::new_v4().to_string();
    let user_id = session.get_user_id().await?;
    let filename = format!("{}.{}", &unique_uuid, extension);
    let lg_fpath = format!("{}/{}/1420-{}", &state.get_tmp_path(), &user_id, &filename);
    let md_fpath = format!("{}/{}/720-{}", &state.get_tmp_path(), &user_id, &filename);
    let sm_fpath = format!("{}/{}/340-{}", &state.get_tmp_path(), &user_id, &filename);
    Ok((filename, lg_fpath, md_fpath, sm_fpath, user_id as u32))
}

pub async fn upload_file(
    session: Session,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    if let Some(metadata_field) = multipart.next_field().await? {
        let metadata_bytes = metadata_field.bytes().await?;
        let metadata_str = std::str::from_utf8(&metadata_bytes)
            .map_err(|_| AppError::BadRequest("Metadata is not valid UTF-8".into()))?;
        let img_metadata: ImageMetadata = serde_json::from_str(metadata_str)
            .map_err(|_| AppError::BadRequest("Invalid metadata JSON".into()))?;

        if let Some(img_field) = multipart.next_field().await? {
            let original_filename = img_field
                .file_name()
                .ok_or_else(|| AppError::BadRequest("Missing filename in upload".into()))?
                .to_string();

            let img_bytes = img_field.bytes().await?;

            // Validate magic bytes before trusting the extension
            if !is_valid_image(&img_bytes) {
                return Err(AppError::BadRequest("File content is not a recognised image".into()));
            }

            let extension = Path::new(&original_filename)
                .extension()
                .and_then(|ext| ext.to_str())
                .ok_or_else(|| AppError::BadRequest("Missing file extension".into()))?
                .to_lowercase();

            if !ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
                return Err(AppError::BadRequest("Unsupported image format".into()));
            }

            let format = ImageFormat::from_extension(&extension)
                .ok_or_else(|| AppError::BadRequest("Unknown image format".into()))?;

            let (filename, lg_fpath, md_fpath, sm_fpath, user_id) =
                create_tmp_path(&state, &session, &extension).await?;

            let cursor = Cursor::new(&img_bytes);
            let dynamic_image = image::load(cursor, format)?;

            let tmp_upload_path = format!("{}/{}", &state.get_tmp_path(), &user_id);
            if !Path::new(&tmp_upload_path).is_dir() {
                std::fs::create_dir(&tmp_upload_path)?;
            }

            let cropped_img = dynamic_image.crop_imm(
                img_metadata.cropImgMd.x,
                img_metadata.cropImgMd.y,
                img_metadata.cropImgMd.width,
                img_metadata.cropImgMd.height,
            );

            let sm_img_height = new_height(&img_metadata);
            let md_img_height = new_height(&img_metadata);
            let lg_img_height = new_height(&img_metadata);

            cropped_img.resize(340, sm_img_height, FilterType::Lanczos3).save(&sm_fpath)?;
            cropped_img.resize(720, md_img_height, FilterType::Lanczos3).save(&md_fpath)?;
            cropped_img.resize(1420, lg_img_height, FilterType::Lanczos3).save(&lg_fpath)?;

            return Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                Json(json!({ "name": filename })),
            ));
        }
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

pub async fn get_blog_file(
    State(state): State<AppState>,
    AxumPath((uid, size, filename)): AxumPath<(i32, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    validate_path_component(&size)?;
    validate_path_component(&filename)?;
    let file_path = format!("{}/{}/{}-{}", state.get_blog_path(), uid, size, filename);
    let f = std::fs::read(&file_path)?;
    Ok((StatusCode::OK, [(header::CONTENT_TYPE, "application/octet-stream")], f))
}

pub async fn get_book_file(
    State(state): State<AppState>,
    AxumPath((uid, size, filename)): AxumPath<(i32, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    validate_path_component(&size)?;
    validate_path_component(&filename)?;
    let file_path = format!("{}/{}/{}-{}", state.get_book_path(), uid, size, filename);
    let f = std::fs::read(&file_path)?;
    Ok((StatusCode::OK, [(header::CONTENT_TYPE, "application/octet-stream")], f))
}

pub async fn get_tmp_file(
    State(state): State<AppState>,
    AxumPath((uid, size, filename)): AxumPath<(i32, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    validate_path_component(&size)?;
    validate_path_component(&filename)?;
    let file_path = format!("{}/{}/{}-{}", state.get_tmp_path(), uid, size, filename);
    let f = std::fs::read(&file_path)?;
    Ok((StatusCode::OK, [(header::CONTENT_TYPE, "application/octet-stream")], f))
}
