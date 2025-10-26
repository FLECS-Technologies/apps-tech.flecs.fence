use crate::state;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::openapi::{RefOr, Schema};
use utoipa::{PartialSchema, ToSchema};

#[derive(Serialize)]
struct Uri(url::Url);

impl PartialSchema for Uri {
    fn schema() -> RefOr<Schema> {
        utoipa::openapi::schema::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::Type::String)
            .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                utoipa::openapi::KnownFormat::Uri,
            )))
            .into()
    }
}

impl ToSchema for Uri {}

#[utoipa::path(
    get,
    path="/meta/issuer",
    responses(
        (status = OK, description = "Issuer that issues auth tokens", body = Uri)
    )
)]
pub async fn get_issuer(State(state): State<state::AppState>) -> Response {
    let issuer = state.issuer.lock().unwrap().url.clone();
    (StatusCode::OK, Json(Uri(issuer))).into_response()
}

#[utoipa::path(
    get,
    path="/meta/jwk",
    responses(
        (status = OK, description = "Jwk that has to be used to verify issued tokens", body = serde_json::Value)
    )
)]
pub async fn get_jwk(State(state): State<state::AppState>) -> Response {
    let jwk = state.issuer.lock().unwrap().jwk.clone();
    (StatusCode::OK, Json(jwk)).into_response()
}
