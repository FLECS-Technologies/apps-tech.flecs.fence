use crate::model::user::SuperAdmin;
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
        (status = NOT_FOUND, description = "No users")
    )
)]
pub async fn get_all(State(state): State<state::AppState>) -> &'static str {
    "404 Not Found"
}

#[utoipa::path(
    get,
    path="/users/{uid}",
    responses(
        (status = OK, description = "Return a single user by its uid")
    ),
    params(
        ("uid" = user::Uid, description = "User ID to query")
    )
)]
pub async fn get(uid: Result<Path<user::Uid>, PathRejection>) -> String {
    if let Err(PathRejection::FailedToDeserializePathParams(_)) = uid {
        return "400 Bad Request".to_string();
    }

    format!("{}", *uid.unwrap())
}

#[utoipa::path(
    put,
    path="/users",
    responses(
        (status = CREATED, description = "Create a new user")
    ),
    request_body(content = user::User)
)]
pub async fn put(user: Result<Json<user::User>, JsonRejection>) -> String {
    format!("{}", 65535)
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
