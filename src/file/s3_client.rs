use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use reqwest::Client;
use serde::Serialize;
use serde_json::json;

use crate::error::AppError;

#[derive(Serialize)]
struct Claims {
    sub: String,
    name: String,
    exp: usize,
}

#[derive(Clone)]
pub struct S3Client {
    http: Client,
    base_url: String,
    jwt_secret: String,
}

impl S3Client {
    pub fn new(base_url: String, jwt_secret: String) -> Self {
        Self { http: Client::new(), base_url, jwt_secret }
    }

    fn make_jwt(&self, user_id: &str) -> Result<String, AppError> {
        let exp = (Utc::now().timestamp() + 3600) as usize;
        let claims = Claims { sub: user_id.to_string(), name: "loony-api".to_string(), exp };
        encode(&Header::default(), &claims, &EncodingKey::from_secret(self.jwt_secret.as_bytes()))
            .map_err(|e| AppError::InternalServerError(e.to_string()))
    }

    /// Create a bucket if it does not already exist, then ensure ACL is public-read-write.
    pub async fn ensure_bucket(&self, bucket: &str) -> Result<(), AppError> {
        let token = self.make_jwt("system")?;
        let create_url = format!("{}/buckets", self.base_url);
        let resp = self.http.post(&create_url)
            .bearer_auth(&token)
            .json(&json!({ "name": bucket, "acl": "public-read-write" }))
            .send()
            .await?;
        let status = resp.status().as_u16();
        if status != 201 && status != 409 {
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!(bucket, status, body, "ensure_bucket unexpected response");
            return Ok(());
        }
        // If bucket already existed, patch its ACL so any authenticated user can write.
        if status == 409 {
            let patch_url = format!("{}/buckets/{}", self.base_url, bucket);
            let patch_resp = self.http.patch(&patch_url)
                .bearer_auth(&token)
                .json(&json!({ "acl": "public-read-write" }))
                .send()
                .await?;
            if !patch_resp.status().is_success() {
                let ps = patch_resp.status().as_u16();
                tracing::warn!(bucket, ps, "ensure_bucket patch acl failed");
            }
        }
        Ok(())
    }

    /// Upload bytes to a bucket/key. Objects are stored with public-read ACL
    /// so they can be served without authentication.
    pub async fn put(
        &self,
        bucket: &str,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
        user_id: &str,
    ) -> Result<(), AppError> {
        let token = self.make_jwt(user_id)?;
        let url = format!("{}/{}/{}", self.base_url, bucket, key);
        let resp = self.http.put(&url)
            .bearer_auth(&token)
            .header("x-acl", "public-read")
            .header("content-type", content_type)
            .body(bytes)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::error!(%bucket, %key, %status, %body, "S3 put failed");
            return Err(AppError::InternalServerError("Failed to store file".into()));
        }
        Ok(())
    }

    /// Delete an object.
    pub async fn delete(&self, bucket: &str, key: &str, user_id: &str) -> Result<(), AppError> {
        let token = self.make_jwt(user_id)?;
        let url = format!("{}/{}/{}", self.base_url, bucket, key);
        let resp = self.http.delete(&url).bearer_auth(&token).send().await?;
        let status = resp.status().as_u16();
        if !resp.status().is_success() && status != 404 {
            tracing::warn!(bucket, key, status, "S3 delete returned unexpected status");
        }
        Ok(())
    }

    /// Copy an object from one bucket/key to another, then delete the source.
    pub async fn mv(
        &self,
        src_bucket: &str,
        src_key: &str,
        dst_bucket: &str,
        dst_key: &str,
        user_id: &str,
    ) -> Result<(), AppError> {
        let (bytes, content_type) = self.get(src_bucket, src_key).await?;
        self.put(dst_bucket, dst_key, bytes, &content_type, user_id).await?;
        self.delete(src_bucket, src_key, user_id).await?;
        Ok(())
    }

    /// Fetch object bytes. Public-read objects need no auth token.
    pub async fn get(&self, bucket: &str, key: &str) -> Result<(Vec<u8>, String), AppError> {
        let url = format!("{}/{}/{}", self.base_url, bucket, key);
        let resp = self.http.get(&url).send().await?;
        if resp.status().as_u16() == 404 {
            return Err(AppError::NotFound("File not found".into()));
        }
        if !resp.status().is_success() {
            tracing::error!(bucket, key, status = %resp.status(), "S3 get failed");
            return Err(AppError::InternalServerError("Failed to retrieve file".into()));
        }
        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let bytes = resp.bytes().await?.to_vec();
        Ok((bytes, content_type))
    }
}
