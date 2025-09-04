use bb8::Pool;
use bb8_postgres::{bb8, PostgresConnectionManager};
use tokio_postgres::NoTls;
use crate::config::{init_env_configs, AppConfig, FileStoragePath};

#[derive(Clone)]
#[allow(dead_code)]
pub struct Dirs {
    tmp_upload: String,
    blog_upload: String,
    book_upload: String,
    user_upload: String
}

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) app_config: AppConfig,
    pub(crate) pg_pool: Pool<PostgresConnectionManager<NoTls>>,
    pub(crate) mailtrap: MailtrapInfo,
    pub(crate) file_storage_path: FileStoragePath 
}

#[derive(Clone)]
pub(crate) struct MailtrapInfo {
    pub(crate) url: String,
    pub(crate) mailtrap_email: String,
    pub(crate) mailtrap_name: Option<String>,
    pub(crate) mailtrap_token_id: String,
}

pub async fn init() -> AppState {
    let configs = init_env_configs().unwrap();

    // *** Postgres connection pool setup
    let pg_manager = PostgresConnectionManager::new_from_stringlike(
        format!(
            "host={} user={} dbname={} password={}",
            configs.pg.hostname, configs.pg.username, configs.pg.dbname, configs.pg.password
        ),
        NoTls,
    )
    .unwrap();
    let pg_pool = Pool::builder().build(pg_manager).await.unwrap();
    let conn = pg_pool.clone();
    let conn = tokio::time::timeout(tokio::time::Duration::from_secs(3), conn.get()).await.expect("Failed to connect to database.");
    let conn = conn.unwrap();
    conn.query("select * from users", &[]).await.unwrap();
    // ***

    // Setup Mailtrap
    let mailtrap_url = format!("https://sandbox.api.mailtrap.io/api/send/{}", configs.mailtrap.sandbox_id);
    let mailtrap = MailtrapInfo { 
        url: mailtrap_url, 
        mailtrap_email: configs.mailtrap.email, 
        mailtrap_name: Some(configs.mailtrap.name),
        mailtrap_token_id: configs.mailtrap.token_id
    };
    
    return AppState {
        app_config: configs.app.clone(),
        pg_pool,
        mailtrap,
        file_storage_path: configs.app.file_storage_path.clone()
    };
}