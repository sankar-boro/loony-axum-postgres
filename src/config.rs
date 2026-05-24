use crate::error::AppError;
use std::env::var;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct FileStoragePath {
    pub(crate) tmp: String,
    pub(crate) user: String,
    pub(crate) blog: String,
    pub(crate) book: String,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct AppConfig {
    pub(crate) app_env: String,
    pub(crate) app_name: String,
    pub(crate) auth_app_name: String,
    pub(crate) hostname: String,
    pub(crate) port: u16,
    // pub(crate) http_port: u16,
    // pub(crate) https_port: u16,
    pub(crate) allowed_origins: String,
    pub(crate) file_storage_path: FileStoragePath,
    pub(crate) secret_key: String,
}

#[derive(Clone, Debug)]
pub(crate) struct PostgresConfig {
    pub(crate) hostname: String,
    pub(crate) username: String,
    pub(crate) dbname: String,
    pub(crate) password: String
}

#[derive(Clone, Debug)]
pub(crate) struct S3Config {
    pub(crate) url: String,
    pub(crate) jwt_secret: String,
}

#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub(crate) app: AppConfig,
    pub(crate) pg: PostgresConfig,
    pub(crate) s3: S3Config,
}

pub fn init_env_configs() -> Result<Config, AppError> {
    let secret_key = var("SECRET_KEY").unwrap();

    let app_env = var("APP_ENV").unwrap();
    let app_name = var("APP_NAME").unwrap();
    let auth_app_name = var("AUTH_APP_NAME").unwrap();

    log::debug!("App Name: {}", app_name);
    let hostname = var("APP_HOSTNAME").unwrap();
    let port = var("PORT").unwrap().parse().unwrap();
    // let http_port = var("APP_HTTP_PORT")?.parse().unwrap();
    // let https_port = var("APP_HTTPS_PORT")?.parse().unwrap();

    let allowed_origins = var("ALLOWED_ORIGINS").unwrap();
    log::debug!("Allowed origins: {}", allowed_origins);
    let pg_hostname = var("PG_HOSTNAME").unwrap();
    let pg_username = var("PG_USERNAME").unwrap();
    let pg_dbname = var("PG_DBNAME").unwrap();
    let pg_password = var("PG_PASSWORD").unwrap();
    let tmp_upload = var("TMP_UPLOADS").unwrap();
    let user_upload = var("USER_UPLOADS").unwrap();
    let blog_upload = var("BLOG_UPLOADS").unwrap();
    let book_upload = var("BOOK_UPLOADS").unwrap();

    let s3_url = var("S3_URL").unwrap();
    let s3_jwt_secret = var("S3_JWT_SECRET").unwrap();

    Ok(Config {
        app: AppConfig {
            app_env,
            app_name,
            auth_app_name,
            secret_key, 
            hostname, 
            port,
            // http_port, 
            // https_port, 
            allowed_origins, 
            file_storage_path: FileStoragePath { 
                tmp: tmp_upload, 
                user: user_upload,
                blog: blog_upload,
                book: book_upload 
            }
        },
        pg: PostgresConfig { 
            hostname: pg_hostname, 
            username: pg_username, 
            dbname: pg_dbname, 
            password: pg_password 
        },
        s3: S3Config { url: s3_url, jwt_secret: s3_jwt_secret },
    })
}