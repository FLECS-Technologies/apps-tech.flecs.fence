use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use tracing::error;

use crate::model::client::{Client, ClientId};

mod versioning;

#[derive(Debug, thiserror::Error)]
pub enum InsertClientError {
    #[error("Client with name '{0}' already exists")]
    DuplicateName(String),
}

#[derive(Debug, thiserror::Error)]
pub enum RemoveClientError {
    #[error("Client with id {0} does not exist")]
    NotFound(ClientId),
    #[error("Client with id {0} is read-only")]
    ReadOnly(ClientId),
}

pub struct ClientDB {
    path: PathBuf,
    clients: HashMap<ClientId, Client>,
    read_only: HashSet<ClientId>,
}

impl ClientDB {
    pub(super) fn new(path: PathBuf, ro_path: PathBuf) -> anyhow::Result<Self> {
        let clients: versioning::ClientStorage = super::load_from_file(path.as_path())?;
        let mut clients: HashMap<ClientId, Client> = clients.into();

        let ro_clients: versioning::ClientStorage = super::load_from_file(ro_path.as_path())?;
        let ro_clients: HashMap<ClientId, Client> = ro_clients.into();

        let mut read_only = HashSet::new();

        for (id, ro_client) in ro_clients {
            if clients.contains_key(&id) {
                anyhow::bail!("Read-only client id {id} conflicts with mutable client id");
            }
            if clients.values().any(|c| c.name == ro_client.name) {
                anyhow::bail!(
                    "Read-only client name '{}' conflicts with mutable client name",
                    ro_client.name
                );
            }
            read_only.insert(id);
            clients.insert(id, ro_client);
        }

        Ok(ClientDB {
            path,
            clients,
            read_only,
        })
    }

    pub fn query_all(&self) -> impl Iterator<Item = &Client> {
        self.clients.values()
    }

    pub fn query_by_id(&self, id: ClientId) -> Option<&Client> {
        self.clients.get(&id)
    }

    pub fn query_by_name(&self, name: &str) -> Option<&Client> {
        self.clients.values().find(|c| c.name == name)
    }

    pub fn is_read_only(&self, id: &ClientId) -> bool {
        self.read_only.contains(id)
    }

    pub fn insert(&mut self, client: Client) -> Result<ClientId, InsertClientError> {
        if self.query_by_name(&client.name).is_some() {
            return Err(InsertClientError::DuplicateName(client.name));
        }
        let id = client.id;
        self.clients.insert(id, client);
        Ok(id)
    }

    pub fn remove(&mut self, id: ClientId) -> Result<(), RemoveClientError> {
        if self.read_only.contains(&id) {
            return Err(RemoveClientError::ReadOnly(id));
        }
        self.clients
            .remove(&id)
            .map(|_| ())
            .ok_or(RemoveClientError::NotFound(id))
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let mutable_clients: HashMap<ClientId, &Client> = self
            .clients
            .iter()
            .filter(|(id, _)| !self.read_only.contains(id))
            .map(|(id, c)| (*id, c))
            .collect();
        super::save_to_file(
            &self.path,
            &versioning::StorageRef::from_refs(&mutable_clients),
        )
    }
}

impl Drop for ClientDB {
    fn drop(&mut self) {
        self.save()
            .unwrap_or_else(|e| error!("Could not persist client database: {e}"));
    }
}
