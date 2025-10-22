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
    pub(crate) app_name: String,
    pub(crate) hostname: String,
    pub(crate) http_port: u16,
    pub(crate) https_port: u16,
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
pub(crate) struct Config {
    pub(crate) app: AppConfig,
    pub(crate) pg: PostgresConfig,
}

pub fn init_env_configs() -> Result<Config, AppError> {
    let secret_key = var("SECRET_KEY")?;

    let app_name = var("APP_NAME")?;
    log::debug!("App Name: {}", app_name);
    let hostname = var("APP_HOSTNAME")?;
    let http_port = var("APP_HTTP_PORT")?.parse()?;
    let https_port = var("APP_HTTPS_PORT")?.parse()?;

    let allowed_origins = var("ALLOWED_ORIGINS")?;
    log::debug!("Allowed origins: {}", allowed_origins);
    let pg_hostname = var("PG_HOSTNAME")?;
    let pg_username = var("PG_USERNAME")?;
    let pg_dbname = var("PG_DBNAME")?;
    let pg_password = var("PG_PASSWORD")?;

    let tmp_upload = var("TMP_UPLOADS")?;
    let user_upload = var("USER_UPLOADS")?;
    let blog_upload = var("BLOG_UPLOADS")?;
    let book_upload = var("BOOK_UPLOADS")?;

    Ok(Config {
        app: AppConfig {
            app_name,
            secret_key, 
            hostname, 
            http_port, 
            https_port, 
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
        }
    })
}