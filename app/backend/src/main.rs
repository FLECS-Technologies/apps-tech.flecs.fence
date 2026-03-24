use tracing_subscriber::EnvFilter;
use user_manager::router::build_router;

use async_signal::{Signal, Signals};
use futures_util::StreamExt;
use tower_http::services::ServeDir;
use user_manager::state;

#[cfg(debug_assertions)]
const DEFAULT_LOG_LEVEL: &str = "debug";
#[cfg(not(debug_assertions))]
const DEFAULT_LOG_LEVEL: &str = "warn";

async fn signal_handler() {
    let mut signals = Signals::new([Signal::Term, Signal::Int]).unwrap();

    while let Some(signal) = signals.next().await {
        if matches!(signal, Ok(Signal::Int) | Ok(Signal::Term)) {
            break;
        }
    }
}

#[tokio::main]
async fn main() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_LEVEL));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let config = user_manager::config::Config::from_env().unwrap();
    let enforcer = state::construct_enforcer(
        config.auth.casbin_model_path.clone(),
        config.auth.casbin_policy_path.clone(),
    )
    .await
    .unwrap();
    let app_state = state::AppState::new(enforcer, &config);
    let router = build_router(app_state).fallback_service(ServeDir::new("./static"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:27000")
        .await
        .unwrap();
    axum::serve(listener, router)
        .with_graceful_shutdown(signal_handler())
        .await
        .unwrap();
}
