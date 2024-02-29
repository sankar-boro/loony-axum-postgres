mod route;
mod auth;

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, State},
    http::{request::Parts, StatusCode},
    routing::get,
    Router,
};
use bb8::{Pool, PooledConnection};
use bb8_postgres::{PostgresConnectionManager, bb8};
use tokio_postgres::NoTls;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use bb8_redis::{RedisConnectionManager, bb8 as bb8redis};
use redis::AsyncCommands;
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub pg_pool: Pool<PostgresConnectionManager<NoTls>>,
    pub redis_pool: bb8redis::Pool<RedisConnectionManager>
}

async fn create_connection() -> AppState {
    // set up connection pool
    let pg_manager = PostgresConnectionManager::new_from_stringlike("host=localhost user=postgres", NoTls)
    .unwrap();
    let pg_pool = Pool::builder().build(pg_manager).await.unwrap();

    tracing::debug!("connecting to redis");
    let redis_manager = RedisConnectionManager::new("redis://:sankar@127.0.0.1:6379/").unwrap();
    let redis_pool = bb8redis::Pool::builder().build(redis_manager).await.unwrap();

    {
        // ping the database before starting
        let mut conn = redis_pool.get().await.unwrap();
        conn.set::<&str, &str, ()>("foo", "bar").await.unwrap();
        let result: String = conn.get("foo").await.unwrap();
        assert_eq!(result, "bar");
    }
    tracing::debug!("successfully connected to redis and pinged it");

    return AppState{
        pg_pool,
        redis_pool
    }
}


#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_tokio_postgres=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let connection = create_connection().await;


    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let router  = route::create_router(connection, cors);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router).await.unwrap();
}

// type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

// async fn get_something(
//     State(pool): State<AppState>,
// ) -> Result<String, (StatusCode, String)> {
//     let conn = pool.pg_pool.get().await.map_err(internal_error)?;

//     let row = conn
//         .query("select user_id, fname, lname from users", &[])
//         .await
//         .map_err(internal_error)?;
//     let user_id: i32 = row[0].try_get(0).map_err(internal_error)?;

//     Ok(user_id.to_string())
// }

// we can also write a custom extractor that grabs a connection from the pool
// which setup is appropriate depends on your application
// struct DatabaseConnection(PooledConnection<'static, PostgresConnectionManager<NoTls>>);

// #[async_trait]
// impl<S> FromRequestParts<S> for DatabaseConnection
// where
//     ConnectionPool: FromRef<S>,
//     S: Send + Sync,
// {
//     type Rejection = (StatusCode, String);

//     async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
//         let pool = ConnectionPool::from_ref(state);

//         let conn = pool.get_owned().await.map_err(internal_error)?;

//         Ok(Self(conn))
//     }
// }

// async fn post_something(
//     DatabaseConnection(conn): DatabaseConnection,
// ) -> Result<String, (StatusCode, String)> {
//     let row = conn
//         .query_one("select 1 + 1", &[])
//         .await
//         .map_err(internal_error)?;
//     let two: i32 = row.try_get(0).map_err(internal_error)?;

//     Ok(two.to_string())
// }

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}


// type RedisConnectionPool = bb8redis::Pool<RedisConnectionManager>;

// async fn redis_connection_pool(
//     State(pool): State<RedisConnectionPool>,
// ) -> Result<String, (StatusCode, String)> {
//     let mut conn = pool.get().await.map_err(internal_error)?;
//     let result: String = conn.get("foo").await.map_err(internal_error)?;
//     Ok(result)
// }

// // we can also write a custom extractor that grabs a connection from the pool
// // which setup is appropriate depends on your application
// struct RedisDatabaseConnection(bb8redis::PooledConnection<'static, RedisConnectionManager>);

// #[async_trait]
// impl<S> FromRequestParts<S> for RedisDatabaseConnection
// where
//     RedisConnectionPool: FromRef<S>,
//     S: Send + Sync,
// {
//     type Rejection = (StatusCode, String);

//     async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
//         let pool = RedisConnectionPool::from_ref(state);

//         let conn = pool.get_owned().await.map_err(internal_error)?;

//         Ok(Self(conn))
//     }
// }