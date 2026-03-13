use crate::model::user;
use crate::model::user::{UpdateUser, UserSummary};
use crate::persist::user_db::{RemoveUserError, UpdateUserError};
use crate::state;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub mod roles;

#[utoipa::path(
    get,
    path="/users/{uid}",
    responses(
        (status = OK, description = "Return a single user by its uid", body = UserSummary),
        (status = NOT_FOUND, description = "User does not exist"),
        (status = BAD_REQUEST, description = "Invalid user ID"),
    ),
    params(
        ("uid" = user::UserId, description = "User ID to query")
    )
)]
pub async fn get(State(state): State<state::AppState>, Path(uid): Path<user::UserId>) -> Response {
    let db = state.db.lock().unwrap();
    match db.users.query_by_uid(uid) {
        Some(user) => Json(UserSummary::from(user)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[utoipa::path(
    patch,
    path="/users/{uid}",
    responses(
        (status = NO_CONTENT, description = "User was updated"),
        (status = NOT_FOUND, description = "User does not exist"),
        (status = CONFLICT, description = "User with that name already exists"),
        (status = BAD_REQUEST, description = "Invalid request body or user ID", body = String),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = String),
    ),
    params(
        ("uid" = user::UserId, description = "User ID to update")
    ),
    request_body(content = UpdateUser)
)]
pub async fn patch(
    State(state): State<state::AppState>,
    Path(uid): Path<user::UserId>,
    Json(update): Json<UpdateUser>,
) -> Response {
    let mut db = state.db.lock().unwrap();
    match db.users.update(uid, update) {
        Ok(()) => {
            if let Err(e) = db.users.save() {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(UpdateUserError::NotFound(_)) => StatusCode::NOT_FOUND.into_response(),
        Err(UpdateUserError::DuplicateName(name)) => (
            StatusCode::CONFLICT,
            format!("User with name '{name}' already exists"),
        )
            .into_response(),
        Err(UpdateUserError::Password(e)) => {
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
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
