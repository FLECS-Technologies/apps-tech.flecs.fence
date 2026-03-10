use std::convert::Infallible;
use std::time::Duration;

use axum::extract::MatchedPath;
use axum::http::Request;
use axum::response::IntoResponse;
use axum::routing::Route;
use tower::Layer;
use tower::Service;
use tower_http::classify::ServerErrorsFailureClass;
use tracing::{Span, info_span};

pub fn logging() -> impl Layer<
    Route,
    Service: Service<
        http::Request<axum::body::Body>,
        Response: IntoResponse + 'static,
        Error: Into<Infallible> + 'static,
        Future: Send + 'static,
    > + Clone
                 + Send
                 + Sync
                 + 'static,
> + Clone
+ Send
+ Sync
+ 'static {
    tower_http::trace::TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            let matched_path = request
                .extensions()
                .get::<MatchedPath>()
                .map(MatchedPath::as_str);
            let path = request.uri().path();
            info_span!(
                "http_request",
                method = ?request.method(),
                matched_path,
                path,
                error = tracing::field::Empty
            )
        })
        .on_request(|req: &Request<_>, _span: &Span| {
            let path = req.uri().path();
            tracing::debug!("request: {} {}", req.method(), path)
        })
        .on_failure(|error: ServerErrorsFailureClass, _, span: &Span| {
            span.record("error", error.to_string());
        })
        .on_response(
            |res: &http::Response<axum::body::Body>, latency: Duration, span: &Span| {
                span.in_scope(|| tracing::debug!("response: {} in {:?}", res.status(), latency))
            },
        )
}
