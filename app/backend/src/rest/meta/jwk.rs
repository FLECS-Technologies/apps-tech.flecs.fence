use crate::state;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[utoipa::path(
    get,
    path="/meta/jwk",
    tag = "Experimental",
    responses(
        (status = OK, description = "Jwk that has to be used to verify issued tokens", body = serde_json::Value)
    )
)]
pub async fn get(State(state): State<state::AppState>) -> Response {
    let jwk = state.issuer.lock().unwrap().jwk.clone();
    (StatusCode::OK, Json(jwk)).into_response()
}
