use crate::model::user::UpdateUser;
use crate::persist::user_db::{RemoveUserError, UpdateUserError};
use crate::state;
use crate::token::Subject;
use axum::Extension;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[utoipa::path(
    patch,
    path="/users/self",
    responses(
        (status = NO_CONTENT, description = "User was updated"),
        (status = UNAUTHORIZED, description = "Not authenticated"),
        (status = CONFLICT, description = "User with that name already exists"),
        (status = BAD_REQUEST, description = "Invalid request body", body = String),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = String),
    ),
    request_body(content = UpdateUser)
)]
pub async fn patch(
    State(state): State<state::AppState>,
    subject: Option<Extension<Subject>>,
    Json(update): Json<UpdateUser>,
) -> Response {
    let Some(Extension(Subject::User(uid))) = subject else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let mut db = state.db.lock().unwrap();
    match db.users.update(uid, update) {
        Ok(()) => {
            if let Err(e) = db.users.save() {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(UpdateUserError::NotFound(_)) => StatusCode::UNAUTHORIZED.into_response(),
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
    path="/users/self",
    responses(
        (status = NO_CONTENT, description = "User was deleted"),
        (status = FORBIDDEN, description = "Cannot delete the super admin"),
        (status = UNAUTHORIZED, description = "Not authenticated"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = String),
    ),
)]
pub async fn delete(
    State(state): State<state::AppState>,
    subject: Option<Extension<Subject>>,
) -> Response {
    let Some(Extension(Subject::User(uid))) = subject else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let mut db = state.db.lock().unwrap();
    match db.users.remove(uid) {
        Ok(()) => {
            if let Err(e) = db.users.save() {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(RemoveUserError::NotFound(_)) => StatusCode::UNAUTHORIZED.into_response(),
        Err(RemoveUserError::SuperAdmin) => StatusCode::FORBIDDEN.into_response(),
    }
}
