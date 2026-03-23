use crate::model::client::{ClientId, ClientSummary};
use crate::persist::client_db::RemoveClientError;
use crate::state;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[utoipa::path(
    get,
    path="/clients/{cid}",
    responses(
        (status = OK, description = "Return a single client", body = ClientSummary),
        (status = NOT_FOUND, description = "Client does not exist"),
        (status = BAD_REQUEST, description = "Invalid client ID"),
    ),
    params(
        ("cid" = String, description = "Client UUID")
    )
)]
pub async fn get(State(state): State<state::AppState>, Path(cid): Path<ClientId>) -> Response {
    let db = state.db.lock().unwrap();
    match db.clients.query_by_id(cid) {
        Some(client) => Json(ClientSummary::from(client)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[utoipa::path(
    delete,
    path="/clients/{cid}",
    responses(
        (status = NO_CONTENT, description = "Client was deleted"),
        (status = NOT_FOUND, description = "Client does not exist"),
        (status = FORBIDDEN, description = "Client is read-only"),
        (status = BAD_REQUEST, description = "Invalid client ID"),
        (status = INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = String),
    ),
    params(
        ("cid" = String, description = "Client UUID")
    )
)]
pub async fn delete(State(state): State<state::AppState>, Path(cid): Path<ClientId>) -> Response {
    let mut db = state.db.lock().unwrap();
    match db.clients.remove(cid) {
        Ok(()) => {
            if let Err(e) = db.clients.save() {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(RemoveClientError::NotFound(_)) => StatusCode::NOT_FOUND.into_response(),
        Err(RemoveClientError::ReadOnly(_)) => StatusCode::FORBIDDEN.into_response(),
    }
}
