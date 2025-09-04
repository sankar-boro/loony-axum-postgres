use crate::error::AppError;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct FileStoragePath {
    pub(crate) tmp: String,
    pub(crate) blog: String,
    pub(crate) book: String,
    pub(crate) user: String,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct AppConfig {
    pub(crate) hostname: String,
    pub(crate) port: String,
    pub(crate) allowed_origins: String,
    pub(crate) file_storage_path: FileStoragePath
}

#[derive(Clone, Debug)]
pub(crate) struct PostgresConfig {
    pub(crate) hostname: String,
    pub(crate) username: String,
    pub(crate) dbname: String,
    pub(crate) password: String
}

#[derive(Clone, Debug)]
pub(crate) struct MailTrapConfig {
    pub(crate) email: String,
    pub(crate) name: String,
    pub(crate) token_id: String,
    pub(crate) sandbox_id: String,
}

#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub(crate) app: AppConfig,
    pub(crate) pg: PostgresConfig,
    pub(crate) mailtrap: MailTrapConfig,
}

pub fn init_env_configs() -> Result<Config, AppError> {

    let hostname = std::env::var("APP_HOSTNAME")?;
    let port = std::env::var("APP_PORT")?;
    let allowed_origins = std::env::var("ALLOWED_ORIGINS")?;
    let pg_hostname = std::env::var("PG_HOSTNAME")?;
    let pg_username = std::env::var("PG_USERNAME")?;
    let pg_dbname = std::env::var("PG_DBNAME")?;
    let pg_password = std::env::var("PG_PASSWORD")?;

    let mailtrap_email = std::env::var("MAILTRAP_EMAIL")?;
    let mailtrap_name = std::env::var("MAILTRAP_NAME")?;
    let mailtrap_token_id = std::env::var("MAILTRAP_TOKEN_ID")?;
    let mailtrap_sandbox_id = std::env::var("MAILTRAP_SANDBOX_ID")?;

    let tmp_upload = std::env::var("TMP_UPLOADS")?;
    let blog_upload = std::env::var("BLOG_UPLOADS")?;
    let book_upload = std::env::var("BOOK_UPLOADS")?;
    let user_upload = std::env::var("USER_UPLOADS")?;

    Ok(Config {
        app: AppConfig { hostname, port, allowed_origins, file_storage_path: FileStoragePath { tmp: tmp_upload, blog: blog_upload, book: book_upload, user: user_upload }},
        pg: PostgresConfig { hostname: pg_hostname, username: pg_username, dbname: pg_dbname, password: pg_password },
        mailtrap: MailTrapConfig { email: mailtrap_email, name: mailtrap_name, token_id: mailtrap_token_id, sandbox_id: mailtrap_sandbox_id }
    })
}