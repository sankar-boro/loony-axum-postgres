use crate::{error::AppError, AppState};
use axum::{
    extract::{Multipart, Path as AxumPath, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use image::{imageops::FilterType, ImageFormat};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Cursor;
use std::path::Path;
use tower_sessions::Session;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct OriImgMd {
    width: i32,
    height: i32,
}

#[derive(Serialize, Deserialize)]
struct CropImgMd {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct ImageMetadata {
    oriImgMd: OriImgMd,
    cropImgMd: CropImgMd,
}

async fn create_tmp_path(
    state: &AppState,
    session: &Session,
    extension: &str,
) -> Result<(String, String, String, String, u32), AppError> {
    let unique_uuid = Uuid::new_v4().to_string();
    let user_id: u32 = session.get("AUTH_USER_ID").await?.unwrap();
    let filename = format!("{}.{}", &unique_uuid, extension);
    let lg_fpath = format!(
        "{}/{}/lg_{}",
        &state.dirs.file_upload_tmp, &user_id, &filename
    );
    let md_fpath = format!(
        "{}/{}/md_{}",
        &state.dirs.file_upload_tmp, &user_id, &filename
    );
    let sm_fpath = format!(
        "{}/{}/sm_{}",
        &state.dirs.file_upload_tmp, &user_id, &filename
    );
    Ok((filename, lg_fpath, md_fpath, sm_fpath, user_id))
}

// Doesn't support image upload size greater then 2mb
pub async fn upload_file(
    session: Session,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    if let Some(metadata_field) = multipart.next_field().await? {
        let metadata_bytes = metadata_field.bytes().await?;
        let metadata_str = std::str::from_utf8(&metadata_bytes).unwrap();
        let img_metadata: ImageMetadata = serde_json::from_str(&metadata_str).unwrap();
        if let Some(img_field) = multipart.next_field().await? {
            let filename = &img_field.file_name().unwrap().to_string();
            let img_bytes = &img_field.bytes().await?;
            // let _ = field.name().unwrap().to_string();
            // let _ = field.content_type().unwrap().to_string();
            let extension = Path::new(&filename)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap();
            let (filename, lg_fpath, md_fpath, sm_fpath, user_id) =
                create_tmp_path(&state, &session, extension).await?;
            let format = ImageFormat::from_extension(extension);
            let cursor = Cursor::new(&img_bytes);
            let dynamic_image = image::load(cursor, format.unwrap())?;

            let tmp_upload_path = &format!("{}/{}", &state.dirs.file_upload_tmp, &user_id);
            if !Path::new(tmp_upload_path).is_dir() {
                std::fs::create_dir(tmp_upload_path)?;
            }

            let cropped_img_lg = dynamic_image.crop_imm(
                img_metadata.cropImgMd.x,
                img_metadata.cropImgMd.y,
                img_metadata.cropImgMd.width,
                img_metadata.cropImgMd.height,
            );
            let sm_img_width = 340;
            let md_img_width = 720;

            let new_width_percent =
                ((img_metadata.cropImgMd.width - 340) * 100) / img_metadata.cropImgMd.width;
            let new_height = img_metadata.cropImgMd.height / 100 * new_width_percent;

            let cropped_img_md =
                cropped_img_lg.resize(md_img_width, new_height, FilterType::Lanczos3);
            let cropped_img_sm =
                cropped_img_lg.resize(sm_img_width, new_height, FilterType::Lanczos3);

            cropped_img_lg.save(&lg_fpath)?;
            cropped_img_md.save(&md_fpath)?;
            cropped_img_sm.save(&sm_fpath)?;
            return Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                Json(json!({
                    "data": {
                        "name": filename
                    }
                })),
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

pub async fn get_file(
    State(state): State<AppState>,
    AxumPath((user_id, size, filename)): AxumPath<(i32, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    // Assuming files are stored in a directory named 'files'
    let file_path = format!(
        "{}/{}/{}/{}",
        &state.dirs.file_upload, user_id, size, filename
    );

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
    AxumPath((user_id, size, filename)): AxumPath<(i32, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    // Assuming files are stored in a directory named 'files'
    let file_path = format!(
        "{}/{}/{}/{}",
        &state.dirs.file_upload, user_id, size, filename
    );
    // Attempt to read the file contents
    let f = std::fs::read(&file_path)?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        f,
    ))
}
