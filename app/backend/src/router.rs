use crate::{rest, state::AppState};
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};

pub fn build_router() -> Router {
    let state = AppState::default();

    Router::new()
        .route("/", get(|| async { "Hello World!" }))
        .route("/login", get(rest::auth::get_login))
        .route("/login", post(rest::auth::post_login))
        .route("/meta/issuer", get(rest::auth::meta::get_issuer))
        .route("/meta/jwk", get(rest::auth::meta::get_jwk))
        .route("/users", get(rest::users::get_all))
        .route(
            "/users/super-admin",
            get(rest::users::get_super_admin).post(rest::users::post_super_admin),
        )
        .route("/users/{uid}", get(rest::users::get))
        .nest("/oauth", crate::oauth::routes::build_router())
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_origin(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}
