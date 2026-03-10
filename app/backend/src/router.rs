use crate::{rest, state::AppState};
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};

mod layer;

pub fn build_router(state: AppState) -> Router {
    let verify_token_middleware =
        axum::middleware::from_fn_with_state(state.clone(), crate::middleware::token::middleware);
    let verify_roles_middleware =
        axum::middleware::from_fn_with_state(state.clone(), crate::middleware::role::middleware);
    Router::new()
        .route("/", get(|| async { "Hello World!" }))
        .route("/login", get(rest::auth::get_login))
        .route("/login", post(rest::auth::post_login))
        .route("/meta/issuer", get(rest::auth::meta::get_issuer))
        .route("/meta/jwk", get(rest::auth::meta::get_jwk))
        .route("/users", get(rest::users::get_all).put(rest::users::put))
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
        .layer(verify_roles_middleware)
        .layer(verify_token_middleware)
        .layer(layer::logging())
        .with_state(state)
}
