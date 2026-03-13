use crate::state;
use axum::response::IntoResponse;
use axum_extra::headers::HeaderMapExt;
use tracing::{debug, error};

pub struct AuthToken(pub Option<String>);

impl<S> axum::extract::FromRequestParts<S> for AuthToken
where
    S: Send + Sync,
{
    type Rejection = http::StatusCode;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;
        type AuthorizationBearerHeader =
            axum_extra::headers::Authorization<axum_extra::headers::authorization::Bearer>;
        match headers.typed_try_get::<AuthorizationBearerHeader>() {
            Ok(Some(axum_extra::headers::Authorization(bearer))) => {
                Ok(AuthToken(Some(bearer.token().to_string())))
            }
            Ok(None) => Ok(AuthToken(None)),
            _ => Err(http::StatusCode::UNAUTHORIZED),
        }
    }
}

pub async fn middleware(
    axum::extract::State(state): axum::extract::State<state::AppState>,
    AuthToken(auth_token): AuthToken,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    if let Some(token) = auth_token.as_deref() {
        let (jwks, issuer) = {
            let issuer = state.issuer.lock().unwrap();
            (
                jsonwebtoken::jwk::JwkSet {
                    keys: vec![issuer.jwk.clone()],
                },
                issuer.url.clone(),
            )
        };
        match crate::token::verify(token, &jwks, &issuer) {
            Err(e) => {
                error!("Failed to verify token: {e}");
                return http::StatusCode::UNAUTHORIZED.into_response();
            }
            Ok((roles, subject)) => {
                debug!(
                    "Successfully verified token of uid {}, roles: {:?}",
                    subject.0, roles.0
                );
                request.extensions_mut().insert(roles);
                request.extensions_mut().insert(subject);
            }
        }
    } else {
        request
            .extensions_mut()
            .insert(crate::token::Roles::default());
    }
    next.run(request).await
}
