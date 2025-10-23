
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};
use time::Duration;

pub struct AppSession {}

impl AppSession {
    pub async fn new(route: &str, inactivity_duration: Duration) -> SessionManagerLayer<RedisStore<RedisPool>> {
        let pool = AppSession::redis(route).await;
        let session_store = RedisStore::new(pool);
        let session_layer = SessionManagerLayer::new(session_store)
            .with_same_site(cookie::SameSite::None)
            .with_secure(true)
            .with_http_only(true)
            .with_expiry(Expiry::OnInactivity(inactivity_duration));
        session_layer
    }

    async fn redis(route: &str) -> RedisPool {
        let pool = RedisPool::new(
            RedisConfig::from_url(route).unwrap(),
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