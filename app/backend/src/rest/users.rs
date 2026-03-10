use crate::model::user::{CreateUser, SuperAdmin, UserId, UserSummary};
use crate::persist::user_db::{InsertUserError, RemoveUserError};
use crate::token::Subject;
use crate::{model::user, state};
use axum::extract::{
    Json, Path, State,
    rejection::{JsonRejection, PathRejection},
};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[utoipa::path(
    get,
    path="/users",
    responses(
        (status = OK, description = "List of all users", body = Vec<UserSummary>),
    )
)]
pub async fn get_all(State(state): State<state::AppState>) -> Json<Vec<UserSummary>> {
    let db = state.db.lock().unwrap();
    let users: Vec<UserSummary> = db.users.query_all().map(UserSummary::from).collect();
    Json(users)
}

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
pub async fn get(uid: Result<Path<user::UserId>, PathRejection>) -> String {
    if let Err(PathRejection::FailedToDeserializePathParams(_)) = uid {
        return "400 Bad Request".to_string();
    }

    format!("{}", *uid.unwrap())
}

#[utoipa::path(
    put,
    path="/users",
    responses(
        (status = CREATED, description = "User was created", body = user::UserId),
        (status = CONFLICT, description = "User with that name already exists"),
        (status = BAD_REQUEST, description = "Invalid request body", body = String),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = String),
    ),
    request_body(content = CreateUser)
)]
pub async fn put(
    State(state): State<state::AppState>,
    user: Result<Json<CreateUser>, JsonRejection>,
) -> Response {
    let user = match user {
        Ok(Json(u)) => u,
        Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };
    let mut db = state.db.lock().unwrap();
    let id = match db.users.insert(user) {
        Ok(id) => id,
        Err(InsertUserError::DuplicateName(name)) => {
            return (
                StatusCode::CONFLICT,
                format!("User with name '{name}' already exists"),
            )
                .into_response();
        }
        Err(InsertUserError::NoAvailableIds) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "No available user IDs".to_string(),
            )
                .into_response();
        }
        Err(InsertUserError::Password(e)) => {
            return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
        }
    };
    if let Err(e) = db.users.save() {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }
    (StatusCode::CREATED, Json(id)).into_response()
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
    uid: Result<Path<user::UserId>, PathRejection>,
) -> Response {
    let uid = match uid {
        Ok(Path(uid)) => uid,
        Err(e) => return (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    };
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
pub async fn delete_self(
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

#[utoipa::path(
    get,
    path="/users/super-admin",
    tag = "Experimental",
    responses(
        (status = NO_CONTENT, description = "Super admin exists"),
        (status = NOT_FOUND, description = "Super admin does not exist"),
    ),
)]
pub async fn get_super_admin(State(state): State<state::AppState>) -> StatusCode {
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
pub async fn post_super_admin(
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
