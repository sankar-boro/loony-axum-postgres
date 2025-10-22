use crate::config::{init_env_configs, Config};
use crate::connections::pg::PgConnection;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) config: Config,
    pub(crate) pg_pool: PgConnection,
}

impl AppState {

    pub fn get_tmp_path(&self) -> &str {
        &self.config.app.file_storage_path.tmp
    }
    pub fn get_blog_path(&self) -> &str {
        &self.config.app.file_storage_path.blog
    }
    pub fn get_book_path(&self) -> &str {
        &self.config.app.file_storage_path.book
    }
    pub fn get_user_path(&self) -> &str {
        &self.config.app.file_storage_path.user
    }
}

pub async fn init_app_state() -> AppState {
    let config = init_env_configs().unwrap();

    let pg_pool = PgConnection::new(&config.pg).await;
    
    return AppState {
        config,
        pg_pool,
    };
}
