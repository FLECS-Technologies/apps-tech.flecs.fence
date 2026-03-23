use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::model::client::AuthMethod;
use crate::state::AppState;
use crate::token;
use axum::extract::{FromRequest, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use oxide_auth::frontends::simple::endpoint::Vacant;
use oxide_auth_axum::OAuthRequest;
use tracing::debug;

pub async fn post(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> impl IntoResponse {
    let (parts, body) = request.into_parts();

    let body_bytes = match axum::body::to_bytes(body, 1024 * 16).await {
        Ok(b) => b,
        Err(_) => return (StatusCode::BAD_REQUEST, "Request body too large").into_response(),
    };

    let form: Vec<(String, String)> = form_urlencoded::parse(&body_bytes).into_owned().collect();
    let grant_type = get_form_value(&form, "grant_type");

    if grant_type == Some("client_credentials") {
        return handle_client_credentials(&state, &parts.headers, &form).into_response();
    }

    // Rebuild request for oxide-auth flow
    let rebuilt = axum::http::Request::from_parts(parts, axum::body::Body::from(body_bytes));
    let oauth_req = match OAuthRequest::from_request(rebuilt, &()).await {
        Ok(r) => r,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid OAuth request").into_response(),
    };

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
    debug!("Triggering access_token_flow()");
    let resp = ep.access_token_flow().execute(oauth_req);
    match resp {
        Ok(r) => r.into_response(),
        Err(e) => {
            debug!("{:#?}", e);
            (StatusCode::BAD_REQUEST, "Invalid OAuth request").into_response()
        }
    }
}

fn get_form_value<'a>(form: &'a [(String, String)], key: &str) -> Option<&'a str> {
    form.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

fn handle_client_credentials(
    state: &AppState,
    headers: &axum::http::HeaderMap,
    form: &[(String, String)],
) -> impl IntoResponse {
    // Try HTTP Basic Auth first, then fall back to body params
    let (client_id_str, client_secret) = match extract_basic_auth(headers) {
        Some((id, secret)) => (id, secret),
        None => {
            let Some(id) = get_form_value(form, "client_id") else {
                return (StatusCode::BAD_REQUEST, "Missing client_id").into_response();
            };
            let Some(secret) = get_form_value(form, "client_secret") else {
                return (StatusCode::BAD_REQUEST, "Missing client_secret").into_response();
            };
            (id.to_string(), secret.to_string())
        }
    };

    let Ok(client_id) = client_id_str.parse() else {
        return (StatusCode::UNAUTHORIZED, "Unknown client").into_response();
    };

    let db = state.db.lock().unwrap();
    let Some(client) = db.clients.query_by_id(client_id) else {
        return (StatusCode::UNAUTHORIZED, "Unknown client").into_response();
    };

    match &client.auth_method {
        AuthMethod::Secret { secret } => {
            if secret.verify(&client_secret).is_err() {
                return (StatusCode::UNAUTHORIZED, "Invalid client secret").into_response();
            }
        }
        AuthMethod::Certificate { .. } => {
            return (
                StatusCode::BAD_REQUEST,
                "Client uses certificate authentication, not secret",
            )
                .into_response();
        }
    }

    let client_groups: Vec<_> = client.groups.iter().cloned().collect();
    let groups = db.groups.query_groups_with_subgroups(&client_groups);
    let roles: Vec<String> = groups.iter().map(|g| g.as_ref().to_string()).collect();
    drop(db);

    let issuer = state.issuer.lock().unwrap();
    let issued = match token::issue_client_token(
        client_id,
        roles,
        issuer.url.clone(),
        issuer.jwk.common.key_id.clone(),
        &issuer.encoding_key,
    ) {
        Ok(t) => t,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    axum::Json(serde_json::json!({
        "access_token": issued.token,
        "token_type": "Bearer",
        "expires_in": 600
    }))
    .into_response()
}

fn extract_basic_auth(headers: &axum::http::HeaderMap) -> Option<(String, String)> {
    let auth = headers.get("authorization")?.to_str().ok()?;
    let encoded = auth.strip_prefix("Basic ")?;
    let decoded = STANDARD.decode(encoded).ok()?;
    let decoded_str = String::from_utf8(decoded).ok()?;
    let (id, secret) = decoded_str.split_once(':')?;
    Some((id.to_string(), secret.to_string()))
}
