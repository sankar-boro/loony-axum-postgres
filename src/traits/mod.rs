use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;

pub trait MoveImages {
    fn move_images(
        &self,
        file_upload_tmp: &str,
        file_upload: &str,
        user_id: i32,
    ) -> Result<(), AppError>;
}

#[derive(Deserialize, Serialize)]
pub struct Images {
    pub name: String,
    pub tags: Vec<String>,
}

impl MoveImages for Vec<Images> {
    fn move_images(
        &self,
        file_upload_tmp: &str,
        file_upload: &str,
        user_id: i32,
    ) -> Result<(), AppError> {
        let mut iter_images = self.iter();
        while let Some(image) = iter_images.next() {
            let source_path = format!("{}/{}/lg_{}", file_upload_tmp, user_id, image.name);
            if std::path::Path::new(&source_path).exists() {
                let destination_path = format!("{}/{}/lg/{}", file_upload, user_id, image.name);
                fs::rename(&source_path, &destination_path)?;
            }

            let source_path = format!("{}/{}/md_{}", file_upload_tmp, user_id, image.name);
            if std::path::Path::new(&source_path).exists() {
                let destination_path = format!("{}/{}/md/{}", file_upload, user_id, image.name);
                fs::rename(&source_path, &destination_path)?;
            }

            let source_path = format!("{}/{}/sm_{}", file_upload_tmp, user_id, image.name);
            if std::path::Path::new(&source_path).exists() {
                let destination_path = format!("{}/{}/sm/{}", file_upload, user_id, image.name);
                fs::rename(&source_path, &destination_path)?;
            }
        }

        Ok(())
    }
}
