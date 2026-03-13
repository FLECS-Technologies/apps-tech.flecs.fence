use crate::model::group::GroupId;
use crate::model::user;
use crate::persist::user_db::{AddGroupError, RemoveGroupError};
use crate::state;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[utoipa::path(
    put,
    path="/users/{uid}/roles/{role}",
    responses(
        (status = NO_CONTENT, description = "Role was assigned to the user"),
        (status = NOT_FOUND, description = "User does not exist"),
        (status = CONFLICT, description = "User already has this role"),
        (status = BAD_REQUEST, description = "Invalid user ID or role"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = String),
    ),
    params(
        ("uid" = user::UserId, description = "User ID to assign the role to"),
        ("role" = GroupId, description = "Role to assign"),
    ),
)]
pub async fn put(
    State(state): State<state::AppState>,
    Path((uid, role)): Path<(user::UserId, GroupId)>,
) -> Response {
    let mut db = state.db.lock().unwrap();
    match db.users.add_group(uid, role) {
        Ok(()) => {
            if let Err(e) = db.users.save() {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(AddGroupError::NotFound(_)) => StatusCode::NOT_FOUND.into_response(),
        Err(AddGroupError::AlreadyAssigned(_)) => StatusCode::CONFLICT.into_response(),
    }
}

#[utoipa::path(
    delete,
    path="/users/{uid}/roles/{role}",
    responses(
        (status = NO_CONTENT, description = "Role was removed from the user"),
        (status = NOT_FOUND, description = "User does not exist or does not have this role"),
        (status = BAD_REQUEST, description = "Invalid user ID or role"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = String),
    ),
    params(
        ("uid" = user::UserId, description = "User ID to remove the role from"),
        ("role" = GroupId, description = "Role to remove"),
    ),
)]
pub async fn delete(
    State(state): State<state::AppState>,
    Path((uid, role)): Path<(user::UserId, GroupId)>,
) -> Response {
    let mut db = state.db.lock().unwrap();
    match db.users.remove_group(uid, &role) {
        Ok(()) => {
            if let Err(e) = db.users.save() {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(RemoveGroupError::NotFound(_)) => StatusCode::NOT_FOUND.into_response(),
        Err(RemoveGroupError::NotAssigned(_)) => StatusCode::NOT_FOUND.into_response(),
    }
}
