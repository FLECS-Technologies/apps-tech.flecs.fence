use crate::{rest, state::AppState};
use axum::{
    Router,
    routing::{get, patch, post, put},
};
use tower_http::cors::{Any, CorsLayer};

mod layer;

pub fn build_router(state: AppState) -> Router {
    let verify_token_middleware =
        axum::middleware::from_fn_with_state(state.clone(), crate::middleware::token::middleware);
    let verify_roles_middleware =
        axum::middleware::from_fn_with_state(state.clone(), crate::middleware::role::middleware);
    Router::new()
        .route("/login", get(rest::login::get))
        .route("/login", post(rest::login::post))
        .route("/meta/issuer", get(rest::meta::issuer::get))
        .route("/meta/jwk", get(rest::meta::jwk::get))
        .route("/users", get(rest::users::get).put(rest::users::put))
        .route(
            "/users/self",
            patch(rest::users::self_::patch).delete(rest::users::self_::delete),
        )
        .route(
            "/users/super-admin",
            get(rest::users::super_admin::get).post(rest::users::super_admin::post),
        )
        .route(
            "/users/{uid}",
            get(rest::users::uid::get)
                .patch(rest::users::uid::patch)
                .delete(rest::users::uid::delete),
        )
        .route(
            "/users/{uid}/roles",
            get(rest::users::uid::roles::get).put(rest::users::uid::roles::put),
        )
        .route(
            "/users/{uid}/roles/{role}",
            put(rest::users::uid::roles::role::put).delete(rest::users::uid::roles::role::delete),
        )
        .route("/oauth/authorize", get(rest::oauth::authorize::get))
        .route("/oauth/token", post(rest::oauth::token::post))
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
