use crate::state;
use crate::token::Roles;
use axum::response::{IntoResponse, Response};
use casbin::CoreApi;
use std::collections::HashSet;
use thiserror::Error;

pub async fn middleware(
    axum::extract::State(state): axum::extract::State<state::AppState>,
    axum::extract::OriginalUri(uri): axum::extract::OriginalUri,
    axum::Extension(Roles(roles)): axum::Extension<Roles>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let role_verification = {
        let enforcer = state.enforcer.lock().unwrap();
        verify_roles(&enforcer, uri.path(), &roles, request.method())
    };
    match role_verification {
        Err(e) => e.into_response(),
        Ok(_) => next.run(request).await,
    }
}

#[derive(Debug, Error)]
enum VerifyRolesError {
    #[error("Action not allowed by permissions")]
    Forbidden,
    #[error(transparent)]
    Casbin(#[from] casbin::Error),
}

impl IntoResponse for VerifyRolesError {
    fn into_response(self) -> Response {
        match self {
            Self::Forbidden => http::StatusCode::FORBIDDEN.into_response(),
            e => (http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
}

fn verify_roles(
    enforcer: &casbin::Enforcer,
    path: &str,
    roles: &HashSet<String>,
    method: &http::method::Method,
) -> Result<(), VerifyRolesError> {
    /// The casbin policy treats '*' as the anonymous/public role everybody has
    const PUBLIC_ROLE: &str = "*";
    for role in std::iter::once(PUBLIC_ROLE).chain(roles.iter().map(|r| r.as_str())) {
        if enforcer
            .enforce((role, path, method.as_str()))
            .map_err(VerifyRolesError::from)?
        {
            return Ok(());
        }
    }
    Err(VerifyRolesError::Forbidden)
}
