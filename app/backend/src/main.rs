use tracing_subscriber::EnvFilter;
use user_manager::router::build_router;

use async_signal::{Signal, Signals};
use futures_util::StreamExt;
use tower_http::services::ServeDir;
use user_manager::state;

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
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env("RUST_LOG"))
        .init();

    let enforcer = state::construct_enforcer().await.unwrap();
    let app_state = state::AppState::new(enforcer);
    let router = build_router(app_state).fallback_service(ServeDir::new("./static"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:27000")
        .await
        .unwrap();
    axum::serve(listener, router)
        .with_graceful_shutdown(signal_handler())
        .await
        .unwrap();
}
