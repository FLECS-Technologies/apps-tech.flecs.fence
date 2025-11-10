use crate::model::session::LoginSession;
use crate::state::AppState;
use axum::extract::{RawQuery, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Redirect};
use cookie::{Cookie, time};
use oxide_auth::endpoint::{OwnerConsent, Solicitation};
use oxide_auth::frontends::simple::endpoint::{FnSolicitor, Vacant};
use oxide_auth_axum::OAuthRequest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RedirectQuery {
    redirect_uri: Option<String>,
}

pub async fn get_authorize(
    State(state): State<AppState>,
    RawQuery(raw_query): RawQuery,
    headers: HeaderMap,
    req: OAuthRequest,
) -> impl IntoResponse {
    /* Try to extract sid from 'Cookie:' headers */
    let sid = headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|h| Cookie::split_parse(h).filter_map(Result::ok))
        .find(|cookie| cookie.name() == "sid")
        .map(|cookie| cookie.value().to_string());

    /* Try to find matching user_session (i.e. user is logged in) */
    let user_sessions = state.user_sessions.lock().unwrap();
    let user_session = match sid {
        Some(ref sid) => user_sessions.get(sid.as_str()),
        None => None,
    };

    /* Either user has no sid or is not logged in -> redirect to login page */
    if sid.is_none() || user_session.is_none() {
        let session = LoginSession::new(raw_query.unwrap());

        let cookie = Cookie::build(("sid", session.get_sid()))
            .path("/")
            .http_only(true)
            .max_age(time::Duration::minutes(5))
            .build();

        let mut set_cookie = HeaderMap::new();
        set_cookie.insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());

        let mut login_sessions = state.login_sessions.lock().unwrap();
        login_sessions.insert(session);

        return (set_cookie, Redirect::to("/login")).into_response();
    }

    let user_session = user_session.unwrap();
    let uid = user_session.get_uid();
    let solicitor = FnSolicitor(
        move |_req: &mut oxide_auth_axum::OAuthRequest, _pre_grant: Solicitation| {
            OwnerConsent::<oxide_auth_axum::OAuthResponse>::Authorized(uid.to_string())
        },
    );

    let mut registrar = state.registrar.lock().unwrap();
    let mut authorizer = state.authorizer.lock().unwrap();
    let mut issuer = state.issuer.lock().unwrap();

    let ep = oxide_auth::frontends::simple::endpoint::Generic {
        registrar: &mut *registrar,
        authorizer: &mut *authorizer,
        issuer: &mut *issuer,
        solicitor,
        scopes: Vacant,
        response: Vacant,
    };
    println!("Triggering authorization_flow()");

    let resp = ep.authorization_flow().execute(req);

    match resp {
        Ok(r) => r.into_response(),
        Err(e) => {
            println!("{:#?}", e);
            (StatusCode::BAD_REQUEST, "Invalid OAuth request").into_response()
        }
    }
}

pub async fn post_token(State(state): State<AppState>, req: OAuthRequest) -> impl IntoResponse {
    let mut registrar = state.registrar.lock().unwrap();
    let mut authorizer = state.authorizer.lock().unwrap();
    let mut issuer = state.issuer.lock().unwrap();

    let ep = oxide_auth::frontends::simple::endpoint::Generic {
        registrar: &mut *registrar,
        authorizer: &mut *authorizer,
        issuer: &mut *issuer,
        solicitor: Vacant,
        scopes: Vacant,
        response: Vacant,
    };
    println!("Triggering access_token_flow()");
    let resp = ep.access_token_flow().execute(req);
    match resp {
        Ok(r) => r.into_response(),
        Err(e) => {
            println!("{:#?}", e);
            (StatusCode::BAD_REQUEST, "Invalid OAuth request").into_response()
        }
    }
}
