
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};
use time::Duration;

pub struct AppSession {}

impl AppSession {
    pub async fn new(inactivity_duration: Duration) -> SessionManagerLayer<RedisStore<RedisPool>> {
        let pool = AppSession::redis().await;
        let session_store = RedisStore::new(pool);
        let session_layer = SessionManagerLayer::new(session_store)
            .with_same_site(cookie::SameSite::None)
            .with_secure(true)
            .with_http_only(true)
            .with_expiry(Expiry::OnInactivity(inactivity_duration));
        session_layer
    }

    async fn redis() -> RedisPool {
        let pool = RedisPool::new(
            RedisConfig::from_url("redis://:sankar@127.0.0.1:6379/").unwrap(),
            None,
            None,
            None,
            6,
        )
        .unwrap();

        pool.connect();
        pool.wait_for_connect().await.unwrap();
        pool
    }
}