pub mod s3_client;

use crate::types::ImageMetadata;
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
use uuid::Uuid;

const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif"];

fn is_valid_image(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\xFF\xD8\xFF")
        || bytes.starts_with(b"\x89PNG\r\n\x1a\n")
        || bytes.starts_with(b"GIF87a")
        || bytes.starts_with(b"GIF89a")
        || (bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP"))
}

fn mime_for_extension(ext: &str) -> &'static str {
    match ext {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

fn encode_image(img: &image::DynamicImage, format: ImageFormat) -> Result<Vec<u8>, AppError> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, format)?;
    Ok(buf.into_inner())
}

pub async fn upload_file(
    axum::extract::Extension(crate::utils::UserId(user_id)): axum::extract::Extension<crate::utils::UserId>,
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

            if !is_valid_image(&img_bytes) {
                return Err(AppError::BadRequest("File content is not a recognised image".into()));
            }

            let extension = Path::new(&original_filename)
                .extension()
                .and_then(|e| e.to_str())
                .ok_or_else(|| AppError::BadRequest("Missing file extension".into()))?
                .to_lowercase();

            if !ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
                return Err(AppError::BadRequest("Unsupported image format".into()));
            }

            let format = ImageFormat::from_extension(&extension)
                .ok_or_else(|| AppError::BadRequest("Unknown image format".into()))?;

            let filename = format!("{}.{}", Uuid::new_v4(), extension);
            let mime = mime_for_extension(&extension);

            let dynamic_image = image::load(Cursor::new(&img_bytes), format)?;
            let cropped = dynamic_image.crop_imm(
                img_metadata.cropImgMd.x,
                img_metadata.cropImgMd.y,
                img_metadata.cropImgMd.width,
                img_metadata.cropImgMd.height,
            );

            let sm = encode_image(&cropped.resize(340, cropped.height(), FilterType::Lanczos3), format)?;
            let md = encode_image(&cropped.resize(720, cropped.height(), FilterType::Lanczos3), format)?;
            let lg = encode_image(&cropped.resize(1420, cropped.height(), FilterType::Lanczos3), format)?;

            let uid_str = user_id.to_string();
            let s3 = state.s3();

            s3.put("tmp", &format!("{}/340-{}", uid_str, filename), sm, mime, &uid_str).await?;
            s3.put("tmp", &format!("{}/720-{}", uid_str, filename), md, mime, &uid_str).await?;
            s3.put("tmp", &format!("{}/1420-{}", uid_str, filename), lg, mime, &uid_str).await?;

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
        Json(json!({ "status": "404", "message": "Image not found" })),
    ))
}

pub async fn get_blog_file(
    State(state): State<AppState>,
    AxumPath((uid, size, filename)): AxumPath<(i32, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let key = format!("{}/{}-{}", uid, size, filename);
    let (bytes, content_type) = state.s3().get("blog", &key).await?;
    Ok((StatusCode::OK, [(header::CONTENT_TYPE, content_type)], bytes))
}

pub async fn get_book_file(
    State(state): State<AppState>,
    AxumPath((uid, size, filename)): AxumPath<(i32, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let key = format!("{}/{}-{}", uid, size, filename);
    let (bytes, content_type) = state.s3().get("book", &key).await?;
    Ok((StatusCode::OK, [(header::CONTENT_TYPE, content_type)], bytes))
}

pub async fn get_tmp_file(
    State(state): State<AppState>,
    AxumPath((uid, size, filename)): AxumPath<(i32, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let key = format!("{}/{}-{}", uid, size, filename);
    let (bytes, content_type) = state.s3().get("tmp", &key).await?;
    Ok((StatusCode::OK, [(header::CONTENT_TYPE, content_type)], bytes))
}
