use crate::config::{init_env_configs, Config};
use crate::connections::pg::PgConnection;
use crate::file::s3_client::S3Client;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) config: Config,
    pub(crate) pg_pool: PgConnection,
    s3_client: S3Client,
}

impl AppState {
    pub fn s3(&self) -> &S3Client {
        &self.s3_client
    }
    pub fn port(&self) -> &u16 {
        &self.config.app.port
    }
    pub fn app_env(&self) -> &str {
        &self.config.app.app_env
    }
}

pub async fn init_app_state() -> AppState {
    let config = init_env_configs().unwrap();
    let pg_pool = PgConnection::new(&config.pg).await;
    let s3_client = S3Client::new(config.s3.url.clone(), config.s3.jwt_secret.clone());

    // Best-effort bucket creation — warns on failure but does not abort startup.
    for bucket in &["tmp", "blog", "book"] {
        if let Err(e) = s3_client.ensure_bucket(bucket).await {
            tracing::warn!(bucket, error = %e, "could not ensure S3 bucket (is loony-s3 running?)");
        }
    }

    AppState { config, pg_pool, s3_client }
}
