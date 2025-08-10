# Cargo.toml

[package]
name = "rust-axum-image-store"
version = "0.2.0"
edition = "2021"

[dependencies]
axum = { version = "0.7", features = ["json", "headers", "multipart"] }
tokio = { version = "1.36", features = ["rt-multi-thread", "macros", "fs"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sled = "0.34"
sha2 = "0.10"
uuid = { version = "1.4", features = ["v4"] }
mime = "0.3"
mime_guess = "2.0"
base64 = "0.21"
image = { version = "0.24", features = ["jpeg", "png", "ico"] }
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
chrono = { version = "0.4", features = ["serde"] }
hmac = "0.12"
sha2 = "0.10"
hex = "0.4"
bytes = "1.4"
tokio-util = "0.8"

# We'll use axum's built-in multipart support (which uses multer under the hood) for streaming uploads.

# src/main.rs

use axum::{
body::{Body, StreamBody},
extract::{Path, RawBody, Multipart},
http::{HeaderMap, HeaderValue, StatusCode, header},
response::{IntoResponse, Response},
routing::{get, post},
Json, Router,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sled::Db;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use uuid::Uuid;
use tracing_subscriber;
use anyhow::Result;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256 as Sha256Hmac;
use hex;
use bytes::Bytes;
use tokio_util::io::ReaderStream;

type HmacSha256 = Hmac<Sha256Hmac>;

#[derive(Clone)]
struct AppState {
db: Arc<Db>,
storage_dir: PathBuf,
signing_key: Vec<u8>, // for pre-signed urls
}

#[derive(Serialize, Deserialize, Clone)]
struct ImageMeta {
id: String,
sha256: String,
filename: Option<String>,
content_type: Option<String>,
size: u64,
uploaded_at: i64,
private: bool,
}

#[derive(Serialize)]
struct UploadResponse {
id: String,
sha256: String,
}

#[tokio::main]
async fn main() -> Result<()> {
tracing_subscriber::fmt::init();

    let storage_dir = std::env::var("STORAGE_DIR").unwrap_or_else(|_| "./storage".to_string());
    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "./metadata.db".to_string());
    let signing_key = std::env::var("SIGNING_KEY").unwrap_or_else(|_| "very-secret-key".to_string()).into_bytes();

    fs::create_dir_all(&storage_dir).await?;

    let db = sled::open(db_path)?;
    let state = AppState {
        db: Arc::new(db),
        storage_dir: PathBuf::from(storage_dir),
        signing_key,
    };

    let app = Router::new()
        .route("/upload", post(upload_multipart_handler))
        .route("/images/:id", get(get_image))
        .route("/images/:id/thumbnail", get(get_thumbnail))
        .route("/sign/:id", get(sign_url))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())

}

// STREAMING MULTIPART UPLOAD
// Accepts multipart form with a field named "file". The file is streamed to disk while computing sha256.
// Optional form fields: filename, private (true|false)
async fn upload_multipart_handler(
mut multipart: Multipart,
state: axum::extract::State<AppState>,
) -> impl IntoResponse {
// we'll iterate fields and find the file
let mut filename: Option<String> = None;
let mut content_type: Option<String> = None;
let mut private = false;
let mut sha_hex = String::new();
let mut total_size: u64 = 0;

    // We'll write to a temporary file first (in the storage dir) streaming chunks.
    // Create temporary path
    let tmp_id = Uuid::new_v4().to_string();
    let mut tmp_path = state.storage_dir.clone();
    tmp_path.push("tmp");
    if let Err(e) = fs::create_dir_all(&tmp_path).await {
        tracing::error!("mkdir tmp failed: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "failed to create tmp dir").into_response();
    }
    tmp_path.push(&tmp_id);

    let mut tmp_file = match File::create(&tmp_path).await {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("tmp file create failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "failed to create tmp file").into_response();
        }
    };

    let mut hasher = Sha256::new();

    while let Some(field_res) = multipart.next_field().await.unwrap_or(None) {
        let name = field_res.name().map(|s| s.to_string());
        if let Some(n) = name.as_deref() {
            match n {
                "file" => {
                    filename = field_res.file_name().map(|s| s.to_string());
                    content_type = field_res.content_type().map(|ct| ct.to_string());
                    // stream the field
                    let mut field_stream = field_res.into_stream();
                    while let Some(chunk_res) = field_stream.next().await {
                        match chunk_res {
                            Ok(bytes) => {
                                total_size += bytes.len() as u64;
                                if let Err(e) = tmp_file.write_all(&bytes).await {
                                    tracing::error!("write tmp failed: {}", e);
                                    let _ = fs::remove_file(&tmp_path).await;
                                    return (StatusCode::INTERNAL_SERVER_ERROR, "failed writing").into_response();
                                }
                                hasher.update(&bytes);
                            }
                            Err(e) => {
                                tracing::error!("read chunk failed: {}", e);
                                let _ = fs::remove_file(&tmp_path).await;
                                return (StatusCode::BAD_REQUEST, "failed reading chunks").into_response();
                            }
                        }
                    }
                }
                "private" => {
                    if let Ok(txt) = field_res.text().await {
                        private = txt.trim().eq_ignore_ascii_case("true");
                    }
                }
                _ => {
                    // ignore other fields
                }
            }
        }
    }

    // finalize
    if let Err(e) = tmp_file.flush().await {
        tracing::warn!("tmp flush failed: {}", e);
    }
    if let Err(e) = tmp_file.sync_all().await {
        tracing::warn!("tmp sync failed: {}", e);
    }

    sha_hex = hex::encode(hasher.finalize());

    // store final path as content-addressed
    let p1 = &sha_hex[0..2];
    let p2 = &sha_hex[2..4];
    let mut final_path = state.storage_dir.clone();
    final_path.push(p1);
    final_path.push(p2);
    if let Err(e) = fs::create_dir_all(&final_path).await {
        tracing::error!("mkdir final failed: {}", e);
        let _ = fs::remove_file(&tmp_path).await;
        return (StatusCode::INTERNAL_SERVER_ERROR, "failed to create dirs").into_response();
    }
    final_path.push(&sha_hex);

    // if already exists, delete tmp; otherwise move
    if fs::metadata(&final_path).await.is_ok() {
        // already exists
        let _ = fs::remove_file(&tmp_path).await;
    } else {
        if let Err(e) = fs::rename(&tmp_path, &final_path).await {
            // fallback to copy
            tracing::warn!("rename failed: {}. Trying copy.", e);
            if let Err(e2) = fs::copy(&tmp_path, &final_path).await {
                tracing::error!("copy failed: {}", e2);
                let _ = fs::remove_file(&tmp_path).await;
                return (StatusCode::INTERNAL_SERVER_ERROR, "failed to store object").into_response();
            }
            let _ = fs::remove_file(&tmp_path).await;
        }
    }

    let id = Uuid::new_v4().to_string();
    let meta = ImageMeta {
        id: id.clone(),
        sha256: sha_hex.clone(),
        filename,
        content_type,
        size: total_size,
        uploaded_at: Utc::now().timestamp(),
        private,
    };

    let key = format!("id:{}", id);
    match serde_json::to_vec(&meta) {
        Ok(data) => {
            if let Err(e) = state.db.insert(key.as_bytes(), data) {
                tracing::error!("db insert failed: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "db insert failed").into_response();
            }
            if let Err(e) = state.db.flush() {
                tracing::warn!("db flush warning: {}", e);
            }
        }
        Err(e) => {
            tracing::error!("serialize meta failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "meta serialize failed").into_response();
        }
    }

    let resp = UploadResponse { id, sha256: sha_hex };
    (StatusCode::OK, Json(resp)).into_response()

}

// RANGE + SIGNED URL support on GET
// If the object is private, the request must either be authenticated (not implemented) or contain valid presigned `sig` & `exp` query params.
async fn get_image(
Path(id): Path<String>,
headers: HeaderMap,
axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
state: axum::extract::State<AppState>,
) -> Response {
let key = format!("id:{}", id);
let value = match state.db.get(key.as_bytes()) {
Ok(Some(v)) => v,
Ok(None) => return (StatusCode::NOT_FOUND, "not found").into_response(),
Err(e) => {
tracing::error!("db get failed: {}", e);
return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
}
};

    let meta: ImageMeta = match serde_json::from_slice(&value) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("meta deserialize failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "meta corrupted").into_response();
        }
    };

    // check privacy + signed URL
    if meta.private {
        // require sig & exp
        match validate_presigned(&id, &params, &state.signing_key) {
            Ok(_) => {}
            Err(msg) => return (StatusCode::UNAUTHORIZED, msg).into_response(),
        }
    }

    let sha = meta.sha256.clone();
    let mut path = state.storage_dir.clone();
    path.push(&sha[0..2]);
    path.push(&sha[2..4]);
    path.push(&sha);

    let file = match File::open(&path).await {
        Ok(f) => f,
        Err(_) => return (StatusCode::NOT_FOUND, "object missing").into_response(),
    };

    let file_len = match file.metadata().await {
        Ok(m) => m.len(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "meta read failed").into_response(),
    };

    // ETag will be the SHA
    let etag = sha.clone();

    // Handle If-None-Match -> 304
    if let Some(inm) = headers.get(header::IF_NONE_MATCH) {
        if let Ok(s) = inm.to_str() {
            if s.trim_matches('"') == etag {
                return (StatusCode::NOT_MODIFIED, "").into_response();
            }
        }
    }

    // Range support
    let mut start: u64 = 0;
    let mut end: u64 = file_len - 1;
    let mut status = StatusCode::OK;
    if let Some(range_hdr) = headers.get(header::RANGE) {
        if let Ok(rstr) = range_hdr.to_str() {
            if rstr.starts_with("bytes=") {
                // only support single range
                if let Some(range_part) = rstr[6..].split(',').next() {
                    let parts: Vec<&str> = range_part.split('-').collect();
                    if parts.len() == 2 {
                        if !parts[0].is_empty() {
                            if let Ok(sv) = parts[0].parse::<u64>() {
                                start = sv;
                            }
                        }
                        if !parts[1].is_empty() {
                            if let Ok(ev) = parts[1].parse::<u64>() {
                                end = ev;
                            }
                        }
                        if start > end || end >= file_len {
                            return (StatusCode::RANGE_NOT_SATISFIABLE, "range not satisfiable").into_response();
                        }
                        status = StatusCode::PARTIAL_CONTENT;
                    }
                }
            }
        }
    }

    let chunk_len = end - start + 1;

    let mut f = file;
    if let Err(e) = f.seek(SeekFrom::Start(start)).await {
        tracing::error!("seek failed: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "seek failed").into_response();
    }

    // Create a reader limited to chunk_len bytes
    let reader = f.take(chunk_len);
    let stream = ReaderStream::new(reader).map_ok(Bytes::from).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));
    let body = StreamBody::new(stream);

    let mut builder = Response::builder().status(status);
    builder = builder.header(header::ACCEPT_RANGES, "bytes");
    builder = builder.header(header::ETAG, format!("\"{}\"", etag));
    builder = builder.header(header::CACHE_CONTROL, "public, max-age=604800, immutable");

    if status == StatusCode::PARTIAL_CONTENT {
        builder = builder.header(header::CONTENT_RANGE, format!("bytes {}-{}/{}", start, end, file_len));
        builder = builder.header(header::CONTENT_LENGTH, chunk_len);
    } else {
        builder = builder.header(header::CONTENT_LENGTH, file_len);
    }

    let content_type = meta
        .content_type
        .clone()
        .or_else(|| mime_guess::from_path(meta.filename.unwrap_or_default()).first_raw().map(|s| s.to_string()))
        .unwrap_or_else(|| "application/octet-stream".to_string());

    builder = builder.header(header::CONTENT_TYPE, content_type);

    builder.body(body).unwrap()

}

// Simple thumbnail endpoint: ?size=200. Thumbnails for private images also require valid signature.
async fn get_thumbnail(
Path(id): Path<String>,
axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
state: axum::extract::State<AppState>,
) -> Response {
let size: u32 = params
.get("size")
.and_then(|s| s.parse().ok())
.unwrap_or(200);

    let key = format!("id:{}", id);
    let value = match state.db.get(key.as_bytes()) {
        Ok(Some(v)) => v,
        Ok(None) => return (StatusCode::NOT_FOUND, "not found").into_response(),
        Err(e) => {
            tracing::error!("db get failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
        }
    };

    let meta: ImageMeta = match serde_json::from_slice(&value) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("meta deserialize failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "meta corrupted").into_response();
        }
    };

    if meta.private {
        match validate_presigned(&id, &params, &state.signing_key) {
            Ok(_) => {}
            Err(msg) => return (StatusCode::UNAUTHORIZED, msg).into_response(),
        }
    }

    let sha = meta.sha256;
    let mut path = state.storage_dir.clone();
    path.push(&sha[0..2]);
    path.push(&sha[2..4]);
    path.push(&sha);

    let bytes = match fs::read(&path).await {
        Ok(b) => b,
        Err(_) => return (StatusCode::NOT_FOUND, "object missing").into_response(),
    };

    match image::load_from_memory(&bytes) {
        Ok(img) => {
            let thumb = img.thumbnail(size, size);
            let mut buf: Vec<u8> = Vec::new();
            if let Err(e) = thumb.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageOutputFormat::Png) {
                tracing::error!("thumbnail encode failed: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "thumbnail encode failed").into_response();
            }

            let mut builder = Response::builder().status(StatusCode::OK);
            builder = builder.header("content-length", buf.len());
            builder = builder.header("content-type", "image/png");
            builder = builder.header("cache-control", "public, max-age=604800, immutable");

            builder.body(Body::from(buf)).unwrap()
        }
        Err(e) => {
            tracing::error!("image decode failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "not an image or decode failed").into_response()
        }
    }

}

// SIGNED URL generation: GET /sign/:id?expiry_seconds=300
// returns JSON { url: "http://.../images/:id?exp=...&sig=..." }
async fn sign_url(
Path(id): Path<String>,
axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
state: axum::extract::State<AppState>,
) -> impl IntoResponse {
let expiry_seconds: u64 = params.get("expiry_seconds").and_then(|s| s.parse().ok()).unwrap_or(300);
let exp = Utc::now().timestamp() + expiry_seconds as i64;
let msg = format!("{}:{}", id, exp);

    let mut mac = HmacSha256::new_from_slice(&state.signing_key).expect("HMAC can take key of any size");
    mac.update(msg.as_bytes());
    let sig = hex::encode(mac.finalize().into_bytes());

    let url = format!("http://127.0.0.1:3000/images/{}?exp={}&sig={}", id, exp, sig);
    let resp = serde_json::json!({"url": url, "expires_at": exp});
    (StatusCode::OK, Json(resp))

}

fn validate*presigned(
id: &str,
params: &std::collections::HashMap<String, String>,
key: &[u8],
) -> Result<(), &'static str> {
let exp_str = params.get("exp").ok_or("missing exp param")?;
let sig = params.get("sig").ok_or("missing sig param")?;
let exp: i64 = exp_str.parse().map_err(|*| "invalid exp")?;
let now = Utc::now().timestamp();
if now > exp {
return Err("signature expired");
}
let msg = format!("{}:{}", id, exp);
let mut mac = HmacSha256::new*from_slice(key).map_err(|*| "invalid key")?;
mac.update(msg.as_bytes());
let expected = hex::encode(mac.finalize().into_bytes());
if &expected != sig {
return Err("invalid signature");
}
Ok(())
}

# README / USAGE (updated)

This version extends the earlier simple example with three features you asked for:

1. **Streaming multipart uploads**: `/upload` now accepts `multipart/form-data` and streams the `file` field to disk while computing its SHA-256. Use an additional form field `private=true` to mark an upload as private. This avoids buffering the whole upload in memory.

   Example using `curl`:

   ```bash
   curl -X POST \
     -F "file=@photo.jpg;type=image/jpeg" \
     -F "private=false" \
     http://127.0.0.1:3000/upload
   ```

2. **Range requests for efficient partial downloads**: `GET /images/:id` supports `Range: bytes=start-end`, returns `206 Partial Content` and `Content-Range` header. Also sets `ETag` (the sha256) for caching and supports `If-None-Match` => `304`.

   Example:

   ```bash
   curl -H "Range: bytes=0-1023" http://127.0.0.1:3000/images/<id> --output part.bin
   ```

3. **Pre-signed short-lived URLs**: `GET /sign/:id?expiry_seconds=300` returns a JSON object with a signed URL valid for `expiry_seconds` (default 300s). Private images require a valid `exp` and `sig` query pair to be accessed.

   Example:

   ```bash
   curl http://127.0.0.1:3000/sign/<id>?expiry_seconds=600
   # => { "url": "http://127.0.0.1:3000/images/<id>?exp=...&sig=...", "expires_at": 1234567890 }
   curl "http://127.0.0.1:3000/images/<id>?exp=...&sig=..." --output out.jpg
   ```

Notes & next steps:

- The presigning uses an HMAC-SHA256 secret (env `SIGNING_KEY`). In production, rotate and protect this key (KMS).
- For large-scale production use: add authentication, quota checks, background thumbnail pre-generation, and an async job queue to avoid on-demand CPU spikes.
- You can switch storage to S3 by writing the streamed bytes to S3 multipart upload instead of the local FS; keep the same content-addressed metadata.
