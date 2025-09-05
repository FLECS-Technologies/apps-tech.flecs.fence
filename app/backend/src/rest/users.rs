use axum::extract::{
    Json, Path, State,
    rejection::{JsonRejection, PathRejection},
};

use crate::{model::user, state};

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
