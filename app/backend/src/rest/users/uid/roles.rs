use std::collections::HashSet;

use crate::model::group::GroupId;
use crate::model::user;
use crate::persist::user_db::SetGroupsError;
use crate::state;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub mod role;

#[utoipa::path(
    get,
    path="/users/{uid}/roles",
    responses(
        (status = OK, description = "List of roles assigned to the user", body = HashSet<GroupId>),
        (status = NOT_FOUND, description = "User does not exist"),
        (status = BAD_REQUEST, description = "Invalid user ID"),
    ),
    params(
        ("uid" = user::UserId, description = "User ID to query roles for")
    ),
)]
pub async fn get(State(state): State<state::AppState>, Path(uid): Path<user::UserId>) -> Response {
    let db = state.db.lock().unwrap();
    match db.users.query_by_uid(uid) {
        Some(user) => Json(&user.groups).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[utoipa::path(
    put,
    path="/users/{uid}/roles",
    responses(
        (status = NO_CONTENT, description = "Roles were assigned to the user"),
        (status = NOT_FOUND, description = "User does not exist"),
        (status = BAD_REQUEST, description = "Invalid request body or user ID"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = String),
    ),
    params(
        ("uid" = user::UserId, description = "User ID to assign roles to")
    ),
    request_body(content = HashSet<GroupId>)
)]
pub async fn put(
    State(state): State<state::AppState>,
    Path(uid): Path<user::UserId>,
    Json(groups): Json<HashSet<GroupId>>,
) -> Response {
    let mut db = state.db.lock().unwrap();
    match db.users.set_groups(uid, groups) {
        Ok(()) => {
            if let Err(e) = db.users.save() {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(SetGroupsError::NotFound(_)) => StatusCode::NOT_FOUND.into_response(),
    }
}
