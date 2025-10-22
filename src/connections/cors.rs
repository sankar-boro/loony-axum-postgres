use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN;
use tower_http::cors::CorsLayer;

pub fn init_cors(allowed_origins: &str) -> CorsLayer {
    // Parse the comma-separated string into a Vec<String>
    let origins: Vec<HeaderValue> = allowed_origins
        .split(',')
        .map(|s| s.parse::<HeaderValue>().unwrap())
        .collect();
    
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE, ACCESS_CONTROL_ALLOW_ORIGIN])
}