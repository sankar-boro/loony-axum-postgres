use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;

pub trait MoveImages {
    fn move_images(&self, file_upload_tmp: &str, file_upload: &str) -> Result<(), AppError>;
}

#[derive(Deserialize, Serialize)]
pub struct Images {
    pub name: String,
}

impl MoveImages for Vec<Images> {
    fn move_images(&self, file_upload_tmp: &str, file_upload: &str) -> Result<(), AppError> {
        let mut iter_images = self.iter();
        while let Some(image) = iter_images.next() {
            let source_path = format!("{}/{}", file_upload_tmp, image.name);
            if std::path::Path::new(&source_path).exists() {
                let destination_path = format!("{}/{}", file_upload, image.name);
                fs::rename(&source_path, &destination_path)?;
            }
        }

        Ok(())
    }
}
