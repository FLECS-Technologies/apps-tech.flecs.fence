use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::model::client::{Client, ClientId};

#[derive(Deserialize)]
#[serde(tag = "version")]
enum StorageEnvelope {
    #[serde(rename = "1")]
    V1 { clients: Vec<Client> },
}

#[derive(Serialize)]
#[serde(tag = "version")]
pub(super) enum StorageRef<'a> {
    #[serde(rename = "1")]
    V1 { clients: Vec<&'a Client> },
}

impl<'a> StorageRef<'a> {
    #[cfg(test)]
    pub(super) fn new(clients: &'a HashMap<ClientId, Client>) -> Self {
        Self::V1 {
            clients: clients.values().collect(),
        }
    }

    pub(super) fn from_refs(clients: &'a HashMap<ClientId, &'a Client>) -> Self {
        Self::V1 {
            clients: clients.values().copied().collect(),
        }
    }
}

#[derive(Default)]
pub(super) struct ClientStorage(pub(super) HashMap<ClientId, Client>);

impl From<ClientStorage> for HashMap<ClientId, Client> {
    fn from(value: ClientStorage) -> Self {
        value.0
    }
}

fn vec_to_map(clients: Vec<Client>) -> HashMap<ClientId, Client> {
    clients.into_iter().map(|c| (c.id, c)).collect()
}

impl<'de> Deserialize<'de> for ClientStorage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        if value.get("version").is_some() {
            let envelope: StorageEnvelope =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            return Ok(match envelope {
                StorageEnvelope::V1 { clients } => ClientStorage(vec_to_map(clients)),
            });
        }

        Err(serde::de::Error::custom(
            "unexpected format for client database",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::client::AuthMethod;
    use crate::model::password::Password;
    use std::collections::HashSet;

    fn test_client() -> Client {
        Client {
            id: uuid::Uuid::new_v4(),
            name: "test-client".to_string(),
            auth_method: AuthMethod::Secret {
                secret: Password::new("TestPassword123!").unwrap(),
            },
            groups: HashSet::new(),
            created_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn deserialize_versioned_v1() {
        let client = test_client();
        let json = serde_json::json!({
            "version": "1",
            "clients": [serde_json::to_value(&client).unwrap()]
        });
        let storage: ClientStorage = serde_json::from_value(json).unwrap();
        assert_eq!(storage.0.len(), 1);
        assert!(storage.0.contains_key(&client.id));
    }

    #[test]
    fn serialize_roundtrip_via_storage_ref() {
        let mut clients = HashMap::new();
        let client = test_client();
        let id = client.id;
        clients.insert(id, client);

        let storage = StorageRef::new(&clients);
        let json = serde_json::to_value(&storage).unwrap();
        assert_eq!(json["version"], "1");
        assert!(json["clients"].is_array());

        let wrapper: ClientStorage = serde_json::from_value(json).unwrap();
        assert_eq!(wrapper.0.len(), 1);
        assert!(wrapper.0.contains_key(&id));
    }

    #[test]
    fn unknown_version_fails() {
        let json = serde_json::json!({
            "version": "999",
            "clients": []
        });
        let result = serde_json::from_value::<ClientStorage>(json);
        assert!(result.is_err());
    }

    #[test]
    fn unexpected_format_fails() {
        let json = serde_json::json!("just a string");
        let result = serde_json::from_value::<ClientStorage>(json);
        assert!(result.is_err());
    }
}
