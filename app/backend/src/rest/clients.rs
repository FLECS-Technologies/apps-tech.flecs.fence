use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand_core::TryRngCore;

use crate::model::client::{
    AuthMethod, Client, ClientSummary, CreateAuthMethod, CreateClient, CreateClientResponse,
};
use crate::model::password::Password;
use crate::persist::client_db::InsertClientError;
use crate::state;
use crate::token::Roles;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub mod cid;

#[utoipa::path(
    get,
    path="/clients",
    responses(
        (status = OK, description = "List of all clients", body = Vec<ClientSummary>),
    )
)]
pub async fn get(State(state): State<state::AppState>) -> Json<Vec<ClientSummary>> {
    let db = state.db.lock().unwrap();
    let clients: Vec<ClientSummary> = db.clients.query_all().map(ClientSummary::from).collect();
    Json(clients)
}

#[utoipa::path(
    put,
    path="/clients",
    responses(
        (status = CREATED, description = "Client was created", body = CreateClientResponse),
        (status = FORBIDDEN, description = "Caller does not have all requested groups", body = String),
        (status = CONFLICT, description = "Client with that name already exists"),
        (status = BAD_REQUEST, description = "Invalid request body", body = String),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = String),
    ),
    request_body(content = CreateClient)
)]
pub async fn put(
    State(state): State<state::AppState>,
    axum::Extension(Roles(caller_roles)): axum::Extension<Roles>,
    Json(create): Json<CreateClient>,
) -> Response {
    let unauthorized_groups: Vec<_> = create
        .groups
        .iter()
        .filter(|g| !caller_roles.contains(g.as_ref()))
        .collect();
    if !unauthorized_groups.is_empty() {
        return (
            StatusCode::FORBIDDEN,
            format!(
                "Cannot assign groups not held by caller: {}",
                unauthorized_groups
                    .iter()
                    .map(|g| g.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        )
            .into_response();
    }

    let (auth_method, secret, certificate, private_key) = match create.auth_method {
        CreateAuthMethod::Secret => {
            let mut secret_bytes = [0u8; 32];
            rand_core::OsRng
                .try_fill_bytes(&mut secret_bytes)
                .expect("OS RNG should work");
            let plaintext_secret = URL_SAFE_NO_PAD.encode(secret_bytes);
            let hashed = match Password::new(&plaintext_secret) {
                Ok(h) => h,
                Err(e) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
                }
            };
            (
                AuthMethod::Secret { secret: hashed },
                Some(plaintext_secret),
                None,
                None,
            )
        }
        CreateAuthMethod::Certificate { .. } => {
            return (
                StatusCode::BAD_REQUEST,
                "Certificate auth method not yet supported".to_string(),
            )
                .into_response();
        }
    };

    let auth_method_name = match &auth_method {
        AuthMethod::Secret { .. } => "secret",
        AuthMethod::Certificate { .. } => "certificate",
    };

    let client = Client {
        id: uuid::Uuid::new_v4(),
        name: create.name,
        auth_method,
        groups: create.groups,
        created_at: chrono::Utc::now(),
    };

    let response = CreateClientResponse {
        id: client.id,
        name: client.name.clone(),
        auth_method: auth_method_name.to_string(),
        groups: client.groups.clone(),
        created_at: client.created_at,
        secret,
        certificate,
        private_key,
    };

    let mut db = state.db.lock().unwrap();
    if let Err(InsertClientError::DuplicateName(name)) = db.clients.insert(client) {
        return (
            StatusCode::CONFLICT,
            format!("Client with name '{name}' already exists"),
        )
            .into_response();
    }
    if let Err(e) = db.clients.save() {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }
    (StatusCode::CREATED, Json(response)).into_response()
}
