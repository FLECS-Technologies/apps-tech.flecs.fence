use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::group::GroupId;
use super::password::Password;

pub type ClientId = uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthMethod {
    Secret { secret: Password },
    Certificate { pem: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Client {
    pub id: ClientId,
    pub name: String,
    pub auth_method: AuthMethod,
    pub groups: HashSet<GroupId>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ClientSummary {
    #[schema(value_type = String)]
    pub id: ClientId,
    pub name: String,
    pub auth_method: String,
    pub groups: HashSet<GroupId>,
    #[schema(value_type = String)]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<&Client> for ClientSummary {
    fn from(client: &Client) -> Self {
        Self {
            id: client.id,
            name: client.name.clone(),
            auth_method: match &client.auth_method {
                AuthMethod::Secret { .. } => "secret".to_string(),
                AuthMethod::Certificate { .. } => "certificate".to_string(),
            },
            groups: client.groups.clone(),
            created_at: client.created_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateClient {
    pub name: String,
    pub auth_method: CreateAuthMethod,
    pub groups: HashSet<GroupId>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum CreateAuthMethod {
    Secret,
    Certificate { pem: Option<String> },
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateClientResponse {
    #[schema(value_type = String)]
    pub id: ClientId,
    pub name: String,
    pub auth_method: String,
    pub groups: HashSet<GroupId>,
    #[schema(value_type = String)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
}
