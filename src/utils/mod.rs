pub mod doc;
pub mod response;

use crate::types::ImageMetadata;

/// User ID injected into request extensions by the `require_auth` middleware.
#[derive(Clone, Copy)]
pub struct UserId(pub i32);

pub fn new_height(img_metadata: &ImageMetadata) -> u32 {
    let new_width_percent =
        ((img_metadata.cropImgMd.width - 340) * 100) / img_metadata.cropImgMd.width;
    let new_height = img_metadata.cropImgMd.height / 100 * new_width_percent;
    new_height
}