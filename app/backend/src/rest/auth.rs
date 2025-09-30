use axum::http::{HeaderMap, StatusCode};
use axum::response::Html;
use axum::{
    extract::{Json, State, rejection::JsonRejection},
    response::{IntoResponse, Redirect},
};
use cookie::Cookie;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::model::password::Password;
use crate::model::session::UserSession;
use crate::state;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    status: String,
}

#[utoipa::path(
    post,
    path="/login",
    responses(
        (status = FOUND, description = "Login successful"),
        (status = NOT_FOUND, description = "No users")
    )
)]
pub async fn post_login(
    State(state): State<state::AppState>,
    headers: HeaderMap,
    payload: Result<Json<LoginRequest>, JsonRejection>,
) -> impl IntoResponse {
    /* validate payload */
    if payload.is_err() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json::from(r##"{"reason":"Invalid request body"}"##),
        ));
    }
    let payload = payload.unwrap();

    /* verify username/password */
    let db = state.db.lock().unwrap();
    let user = db.users.query_by_name(&payload.username);
    if user.is_none() {
        return Err((
            StatusCode::FORBIDDEN,
            Json::from(r##"{"reason":"Invalid username and/or password"}"##),
        ));
    }

    let user = user.unwrap();
    if let Err(_) = user.password.verify(&payload.password) {
        return Err((
            StatusCode::FORBIDDEN,
            Json::from(r##"{"reason":"Invalid username and/or password"}"##),
        ));
    }

    /* login successful, remove login session, if any */
    let mut login_sessions = state.login_sessions.lock().unwrap();
    let sid = extract_sid_from_request_headers(&headers);
    let login_session = match sid {
        Some(s) => login_sessions.take(s.as_str()),
        None => None,
    };

    /* create new user-session and tie it to the user's uid */
    /* @todo add granted scope to user session */
    let user_session = UserSession::new(user.uid.clone());

    let cookie = Cookie::build(("sid", user_session.get_sid()))
        .path("/")
        .http_only(true)
        .build();

    let mut set_cookie = HeaderMap::new();
    set_cookie.insert(
        axum::http::header::SET_COOKIE,
        cookie.to_string().parse().unwrap(),
    );

    let mut user_sessions = state.user_sessions.lock().unwrap();
    user_sessions.insert(user_session);

    match login_session {
        Some(s) => Ok((
            set_cookie,
            Redirect::to(format!("/oauth/authorize?{}", s.get_q()).as_str()).into_response(),
        )),
        None => Ok((set_cookie, Html("Login successful").into_response())),
    }
}

fn extract_sid_from_request_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(axum::http::header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|raw| Cookie::split_parse(raw).filter_map(Result::ok))
        .find(|cookie| cookie.name() == "sid")
        .map(|cookie| cookie.value().to_string())
}

fn has_login_session(headers: &HeaderMap, sid: &str) -> bool {
    headers
        .get_all(axum::http::header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|raw| Cookie::split_parse(raw).filter_map(Result::ok))
        .any(|cookie| cookie.name() == "sid" && cookie.value() == sid)
}

fn has_valid_session(headers: &HeaderMap) -> bool {
    headers
        .get_all(axum::http::header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|raw| Cookie::split_parse(raw).filter_map(Result::ok))
        .any(|cookie| cookie.name() == "sid")
}
