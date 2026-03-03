use user_manager::router::build_router;

use async_signal::{Signal, Signals};
use futures_util::StreamExt;
use tower_http::services::ServeDir;

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
    let router = build_router().fallback_service(ServeDir::new("./static"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:27000")
        .await
        .unwrap();
    axum::serve(listener, router)
        .with_graceful_shutdown(signal_handler())
        .await
        .unwrap();
}
