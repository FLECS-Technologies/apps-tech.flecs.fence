use std::collections::HashMap;
use std::path::PathBuf;
use tracing::error;

use crate::model::password::{self, HashError};
use crate::model::user::{CreateUser, SUPER_ADMIN_ID, SuperAdmin, User, UserId};

mod versioning;

#[derive(Debug, thiserror::Error)]
pub enum InsertUserError {
    #[error("User with name '{0}' already exists")]
    DuplicateName(String),
    #[error("No available user IDs")]
    NoAvailableIds,
    #[error("Invalid password: {0}")]
    Password(#[from] HashError),
}

pub struct UserDB {
    path: PathBuf,
    users: HashMap<UserId, User>,
}

impl UserDB {
    pub(super) fn new(path: PathBuf) -> anyhow::Result<Self> {
        let users: versioning::UserStorage = super::load_from_file(path.as_path())?;
        let users = users.into();
        Ok(UserDB { path, users })
    }

    pub fn query_by_name(&self, name: &str) -> Option<&User> {
        self.users.values().find(|u| u.name == name)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        super::save_to_file(&self.path, &versioning::StorageRef::new(&self.users))
    }

    pub fn query_by_uid(&self, uid: UserId) -> Option<&User> {
        self.users.get(&uid)
    }

    pub fn contains_super_admin(&self) -> bool {
        self.users.contains_key(&SUPER_ADMIN_ID)
    }

    pub fn insert(&mut self, create: CreateUser) -> Result<UserId, InsertUserError> {
        if self.query_by_name(&create.name).is_some() {
            return Err(InsertUserError::DuplicateName(create.name));
        }
        let id = (1..=UserId::MAX)
            .find(|id| !self.users.contains_key(id))
            .ok_or(InsertUserError::NoAvailableIds)?;
        let user = User {
            id,
            name: create.name,
            full_name: String::new(),
            password: password::Password::new(&create.password)?,
            groups: create.groups,
        };
        self.users.insert(id, user);
        Ok(id)
    }

    pub fn set_super_admin(&mut self, super_admin: SuperAdmin) -> anyhow::Result<Option<User>> {
        let super_admin: User = super_admin.try_into()?;
        Ok(self.users.insert(SUPER_ADMIN_ID, super_admin))
    }
}

impl Drop for UserDB {
    fn drop(&mut self) {
        self.save()
            .unwrap_or_else(|e| error!("Could not persist user database: {e}"));
    }
}
