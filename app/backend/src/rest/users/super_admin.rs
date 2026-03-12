use crate::model::user::SuperAdmin;
use crate::state;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[utoipa::path(
    get,
    path="/users/super-admin",
    tag = "Experimental",
    responses(
        (status = NO_CONTENT, description = "Super admin exists"),
        (status = NOT_FOUND, description = "Super admin does not exist"),
    ),
)]
pub async fn get(State(state): State<state::AppState>) -> StatusCode {
    if state.db.lock().unwrap().users.contains_super_admin() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

#[utoipa::path(
    post,
    path="/users/super-admin",
    tag = "Experimental",
    request_body(
        content = SuperAdmin,
        description = "The super admin that should be set",
    ),
    responses(
        (status = OK, description = "Super admin was created"),
        (status = CONFLICT, description = "Super admin already exists"),
        (status = BAD_REQUEST, description = "Invalid password", body = String),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = String),
    ),
)]
pub async fn post(
    State(state): State<state::AppState>,
    Json(super_admin): Json<SuperAdmin>,
) -> Response {
    let mut db = state.db.lock().unwrap();
    if db.users.contains_super_admin() {
        return StatusCode::CONFLICT.into_response();
    }
    if let Err(e) = db.users.set_super_admin(super_admin) {
        return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
    };
    if let Err(e) = db.users.save() {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    };
    StatusCode::OK.into_response()
}
