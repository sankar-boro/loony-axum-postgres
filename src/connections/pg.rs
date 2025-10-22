use bb8::{Pool, PooledConnection};
use bb8_postgres::{bb8, PostgresConnectionManager};
use tokio_postgres::{types::ToSql, Config, NoTls, Row};
use crate::{config::PostgresConfig, error::AppError};

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
        test_conn.query("select * from users", &[]).await.unwrap();
        
        PgConnection {
            conn: conn.clone()
        }
    }

    pub async fn query_opt(&self, query: &str, params: &[&(dyn ToSql + Sync)]) -> Result<Option<Row>, AppError> {
        let conn: PooledConnection<'_, PostgresConnectionManager<NoTls>> = self.conn.get().await?;
        let row: Option<Row> = conn.query_opt(query, params).await?;
        Ok(row)
    }

    pub async fn query_one(&self, query: &str, params: &[&(dyn ToSql + Sync)]) -> Result<Row, AppError> {
        let conn: PooledConnection<'_, PostgresConnectionManager<NoTls>> = self.conn.get().await?;
        let row: Row = conn.query_one(query, params).await?;
        Ok(row)
    }

    pub async fn execute(&self, query: &str, params: &[&(dyn ToSql + Sync)]) -> Result<u64, AppError> {
        let conn: PooledConnection<'_, PostgresConnectionManager<NoTls>> = self.conn.get().await?;
        let row: u64 = conn.execute(query, params).await?;
        Ok(row)
    }

    pub async fn transaction(&self, query: &str, params: &[&(dyn ToSql + Sync)]) -> Result<u64, AppError> {
        let conn: PooledConnection<'_, PostgresConnectionManager<NoTls>> = self.conn.get().await?;
        let row: u64 = conn.execute(query, params).await?;
        Ok(row)
    }
}