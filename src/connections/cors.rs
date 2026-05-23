use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use tower_http::cors::CorsLayer;

pub fn init_cors(allowed_origins: &str) -> CorsLayer {

    let origins: Vec<HeaderValue> = allowed_origins
        .split(';')
        .flat_map(|entry| {
            let mut parts = entry.split(':');
            let port = parts.next().unwrap();
            let protocols = parts.next().unwrap().split(',');

            protocols.map(move |p| format!("{p}://localhost:{port}"))
        })
        .filter_map(|s| HeaderValue::from_str(&s).ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
}