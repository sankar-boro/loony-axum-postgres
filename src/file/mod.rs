use axum::{
    extract::Multipart,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;

// Doesn't support image upload size greater then 2mb
pub async fn upload_file(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let file_name = field.file_name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        println!(
            "Length of `{name}` (`{file_name}`: `{content_type}`) is {} bytes",
            data.len()
        );
    }

    let new_book = json!({
        "status": 200,
        "message": "Uploaded"
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        Json(new_book),
    ))
}
