use crate::error::AppError;
use crate::file::s3_client::S3Client;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Images {
    pub name: String,
}

/// Move every image in `images` from `{src_bucket}/{user_id}/{size}-{name}`
/// to `{dst_bucket}/{project_id}/{size}-{name}` via the S3 service.
/// Errors are logged but not propagated — mirrors the previous best-effort
/// filesystem rename behaviour (callers use `let _ = ...`).
pub async fn move_images_to_s3(
    images: &[Images],
    s3: &S3Client,
    src_bucket: &str,
    dst_bucket: &str,
    user_id: i32,
    project_id: i32,
) -> Result<(), AppError> {
    for image in images {
        for size in &[340u32, 720, 1420] {
            let src_key = format!("{}/{}-{}", user_id, size, image.name);
            let dst_key = format!("{}/{}-{}", project_id, size, image.name);
            if let Err(e) = s3.mv(src_bucket, &src_key, dst_bucket, &dst_key, &user_id.to_string()).await {
                tracing::warn!(src_bucket, %src_key, dst_bucket, %dst_key, error = %e, "move_images_to_s3 failed for one image");
            }
        }
    }
    Ok(())
}
