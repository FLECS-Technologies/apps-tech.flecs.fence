use axum::Router;
use axum::routing::get;

use crate::state::AppState;

pub fn build_router() -> axum::Router<AppState> {
    Router::new().route("/authorize", get(super::authorize::get_authorize))
}
