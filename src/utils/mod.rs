pub mod doc;
pub mod data;
pub mod response;

use crate::error::AppError;
use crate::types::ImageMetadata;
use tower_sessions::Session;
use uuid::Uuid;

pub trait GetUserId {
    async fn get_user_id(&self) -> Result<i32, AppError>;
}

impl GetUserId for Session {
    async fn get_user_id(&self) -> Result<i32, AppError> {
        let user_id: i32 = match self.get("user_id").await {
            Ok(x) => match x {
                Some(x) => x,
                None => {
                    return Err(AppError::InternalServerError(
                        "User session not found".to_string(),
                    ))
                }
            },
            Err(e) => return Err(AppError::InternalServerError(e.to_string())),
        };
        Ok(user_id)
    }
}

pub fn new_height(img_metadata: &ImageMetadata) -> u32 {
    let new_width_percent =
        ((img_metadata.cropImgMd.width - 340) * 100) / img_metadata.cropImgMd.width;
    let new_height = img_metadata.cropImgMd.height / 100 * new_width_percent;
    new_height
}

pub fn get_new_uuid_v4() -> String {
    Uuid::new_v4().to_string()
}