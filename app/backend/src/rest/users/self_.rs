use crate::model::user::UserId;
use crate::persist::user_db::RemoveUserError;
use crate::state;
use crate::token::Subject;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

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
    request: axum::extract::Request,
) -> Response {
    let Some(subject) = request.extensions().get::<Subject>() else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Ok(uid) = subject.0.parse::<UserId>() else {
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
