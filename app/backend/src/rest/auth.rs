use axum::http::HeaderMap;
use axum::response::Html;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};
use cookie::Cookie;

use crate::model::session::UserSession;
use crate::state;

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
) -> impl IntoResponse {
    /* Query if a login session is available */
    let login_session = {
        if let Some(sid) = extract_sid_from_request_headers(&headers) {
            let mut login_sessions = state.login_sessions.lock().unwrap();
            login_sessions.take(sid.as_str())
        } else {
            None
        }
    };

    /* add username/password check here, determine uid and attach to user_session */
    let user_session = UserSession::new();

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
        Some(s) => (
            set_cookie,
            Redirect::to(format!("/oauth/authorize?{}", s.get_q()).as_str()).into_response(),
        ),
        None => (set_cookie, Html("Login successful").into_response()),
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
