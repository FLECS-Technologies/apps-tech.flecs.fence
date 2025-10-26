use axum::Router;
use axum::routing::{get, post};

use crate::state::AppState;

pub fn build_router() -> axum::Router<AppState> {
    Router::new()
        .route("/authorize", get(super::authorize::get_authorize))
        .route("/token", post(super::authorize::post_token))
}
