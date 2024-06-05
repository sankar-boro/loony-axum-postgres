use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;

pub trait MoveImages {
    fn move_images(
        &self,
        file_upload_tmp: &str,
        file_upload: &str,
        user_id: i32,
        project_id: i32,
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
        project_id: i32,
    ) -> Result<(), AppError> {
        let mut iter_images = self.iter();
        let project_path = format!("{}/{}", file_upload, project_id);
        if std::path::Path::new(&project_path).exists() {
            fs::create_dir(&project_path)?;
        }

        while let Some(image) = iter_images.next() {
            if std::path::Path::new(&project_path).exists() {
                let source_path = format!("{}/{}/lg_{}", file_upload_tmp, user_id, image.name);
                let destination_path = format!("{}/{}/lg/{}", file_upload, project_id, image.name);
                fs::rename(&source_path, &destination_path)?;
            }

            if std::path::Path::new(&project_path).exists() {
                let source_path = format!("{}/{}/md_{}", file_upload_tmp, user_id, image.name);
                let destination_path = format!("{}/{}/md/{}", file_upload, project_id, image.name);
                fs::rename(&source_path, &destination_path)?;
            }

            if std::path::Path::new(&project_path).exists() {
                let source_path = format!("{}/{}/sm_{}", file_upload_tmp, user_id, image.name);
                let destination_path = format!("{}/{}/sm/{}", file_upload, project_id, image.name);
                fs::rename(&source_path, &destination_path)?;
            }
        }

        Ok(())
    }
}
