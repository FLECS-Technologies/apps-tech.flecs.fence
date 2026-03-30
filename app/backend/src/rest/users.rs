use crate::model::user;
use crate::model::user::{CreateUser, UserSummary};
use crate::persist::user_db::InsertUserError;
use crate::state;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub mod self_;
pub mod super_admin;
pub mod uid;

#[utoipa::path(
    get,
    path="/users",
    responses(
        (status = OK, description = "List of all users", body = Vec<UserSummary>),
    )
)]
pub async fn get(State(state): State<state::AppState>) -> Json<Vec<UserSummary>> {
    let db = state.db.lock().unwrap();
    let users: Vec<UserSummary> = db.users.query_all().map(UserSummary::from).collect();
    Json(users)
}

#[utoipa::path(
    post,
    path="/users",
    responses(
        (status = CREATED, description = "User was created", body = user::UserId),
        (status = CONFLICT, description = "User with that name already exists"),
        (status = BAD_REQUEST, description = "Invalid request body", body = String),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = String),
    ),
    request_body(content = CreateUser)
)]
pub async fn post(State(state): State<state::AppState>, Json(user): Json<CreateUser>) -> Response {
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
