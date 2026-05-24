use bb8::Pool;
use bb8_postgres::{bb8, PostgresConnectionManager};
use tokio_postgres::{Config, NoTls};
use crate::config::PostgresConfig;

#[derive(Clone)]
pub struct PgConnection {
    pub(crate) conn: Pool<PostgresConnectionManager<NoTls>>
}

impl PgConnection {
    pub async fn new(config: &PostgresConfig) -> Self {
        let mut cfg = Config::new();
        cfg.dbname(&config.dbname);
        cfg.user(&config.username);
        cfg.host(&config.hostname);
        cfg.password(&config.password);

        let pg_manager = PostgresConnectionManager::new(
            cfg,
            NoTls,
        );
    
        let pg_pool = Pool::builder()
        .build(pg_manager).await
        .unwrap();

        let conn = pg_pool.clone();

        let test_conn = tokio::time::timeout(tokio::time::Duration::from_secs(3), conn.get()).await.expect("Failed to connect to database.");
        let test_conn = test_conn.unwrap();
        test_conn.query("select * from blogs", &[]).await.unwrap();
        tracing::info!("Successfully connected to Postgres database.");
        
        PgConnection {
            conn: conn.clone()
        }
    }

}