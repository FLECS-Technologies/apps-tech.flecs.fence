use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use oxide_auth::frontends::simple::endpoint::Vacant;
use oxide_auth_axum::OAuthRequest;
use tracing::debug;

pub async fn post(State(state): State<AppState>, req: OAuthRequest) -> impl IntoResponse {
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
    let resp = ep.access_token_flow().execute(req);
    match resp {
        Ok(r) => r.into_response(),
        Err(e) => {
            debug!("{:#?}", e);
            (StatusCode::BAD_REQUEST, "Invalid OAuth request").into_response()
        }
    }
}
