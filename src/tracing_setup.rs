use tracing_subscriber::{fmt, EnvFilter, Registry};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init_tracing() {
    let app_name = std::env::var("APP_NAME").unwrap_or("loony-auth".to_string());
    let default_filter = format!("{}=DEBUG", app_name);

    // Try to read the RUST_LOG env var, otherwise use the provided default filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_filter));

    // Build the subscriber (registry + layers)
    let subscriber = Registry::default()
        .with(env_filter)
        .with(fmt::layer());

    // Set it as global subscriber (can only be done once)
    subscriber.init();
}