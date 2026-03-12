use crate::model::user;
use crate::persist::user_db::RemoveUserError;
use crate::state;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[utoipa::path(
    get,
    path="/users/{uid}",
    responses(
        (status = OK, description = "Return a single user by its uid")
    ),
    params(
        ("uid" = user::UserId, description = "User ID to query")
    )
)]
pub async fn get(State(state): State<state::AppState>, Path(uid): Path<user::UserId>) -> String {
    format!("{}", uid)
}

#[utoipa::path(
    delete,
    path="/users/{uid}",
    responses(
        (status = NO_CONTENT, description = "User was deleted"),
        (status = NOT_FOUND, description = "User does not exist"),
        (status = FORBIDDEN, description = "Cannot delete the super admin"),
        (status = BAD_REQUEST, description = "Invalid user ID"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = String),
    ),
    params(
        ("uid" = user::UserId, description = "User ID to delete")
    )
)]
pub async fn delete(
    State(state): State<state::AppState>,
    Path(uid): Path<user::UserId>,
) -> Response {
    let mut db = state.db.lock().unwrap();
    match db.users.remove(uid) {
        Ok(()) => {
            if let Err(e) = db.users.save() {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(RemoveUserError::NotFound(_)) => StatusCode::NOT_FOUND.into_response(),
        Err(RemoveUserError::SuperAdmin) => StatusCode::FORBIDDEN.into_response(),
    }
}
