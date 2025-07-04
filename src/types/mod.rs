use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
pub struct OriImgMd {
    pub width: i32,
    pub height: i32,
}

#[derive(Serialize, Deserialize)]
pub struct CropImgMd {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct ImageMetadata {
    pub oriImgMd: OriImgMd,
    pub cropImgMd: CropImgMd,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Book {
    pub uid: i32,
    pub user_id: i32,
    pub title: String,
    pub images: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Blog {
    pub uid: i32,
    pub user_id: i32,
    pub title: String,
    pub images: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct BookNode {
    pub uid: i32,
    pub doc_id: i32,
    pub parent_id: Option<i32>,
    pub title: String,
    pub content: String,
    pub images: Option<String>,
    pub identity: i16,
    pub page_id: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ChildNode {
    pub uid: i32,
    pub parent_id: i32,
    pub title: String,
    pub content: String,
    pub images: Option<String>,
    pub identity: i16,
    pub page_id: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct NavNodes {
    pub uid: i32,
    pub parent_id: Option<i32>,
    pub title: String,
    pub content: Option<String>,
    pub identity: i16,
    pub page_id: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct BookParentNode {
    pub uid: i32,
    pub user_id: i32,
    pub doc_id: i32,
    pub title: String,
    pub content: String,
    pub images: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct BlogNode {
    pub uid: i32,
    pub doc_id: i32,
    pub parent_id: Option<i32>,
    pub title: String,
    pub content: String,
    pub images: Option<String>,
    pub created_at: DateTime<Utc>,
}