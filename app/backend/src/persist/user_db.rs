use std::collections::HashMap;
use std::path::PathBuf;
use tracing::error;

use crate::model::group::GroupId;
use crate::model::password::{self, HashError};
use crate::model::user::{CreateUser, SUPER_ADMIN_ID, SuperAdmin, UpdateUser, User, UserId};

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

#[derive(Debug, thiserror::Error)]
pub enum RemoveUserError {
    #[error("User with id {0} does not exist")]
    NotFound(UserId),
    #[error("Cannot delete the super admin")]
    SuperAdmin,
}

#[derive(Debug, thiserror::Error)]
pub enum SetGroupsError {
    #[error("User with id {0} does not exist")]
    NotFound(UserId),
}

#[derive(Debug, thiserror::Error)]
pub enum AddGroupError {
    #[error("User with id {0} does not exist")]
    NotFound(UserId),
    #[error("User already has role {0}")]
    AlreadyAssigned(GroupId),
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateUserError {
    #[error("User with id {0} does not exist")]
    NotFound(UserId),
    #[error("User with name '{0}' already exists")]
    DuplicateName(String),
    #[error("Invalid password: {0}")]
    Password(#[from] HashError),
}

#[derive(Debug, thiserror::Error)]
pub enum RemoveGroupError {
    #[error("User with id {0} does not exist")]
    NotFound(UserId),
    #[error("User does not have role {0}")]
    NotAssigned(GroupId),
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

    pub fn query_all(&self) -> impl Iterator<Item = &User> {
        self.users.values()
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

    pub fn remove(&mut self, uid: UserId) -> Result<(), RemoveUserError> {
        if uid == SUPER_ADMIN_ID {
            return Err(RemoveUserError::SuperAdmin);
        }
        self.users
            .remove(&uid)
            .map(|_| ())
            .ok_or(RemoveUserError::NotFound(uid))
    }

    pub fn set_groups(
        &mut self,
        uid: UserId,
        groups: std::collections::HashSet<GroupId>,
    ) -> Result<(), SetGroupsError> {
        let user = self
            .users
            .get_mut(&uid)
            .ok_or(SetGroupsError::NotFound(uid))?;
        user.groups = groups;
        Ok(())
    }

    pub fn add_group(&mut self, uid: UserId, group: GroupId) -> Result<(), AddGroupError> {
        let user = self
            .users
            .get_mut(&uid)
            .ok_or(AddGroupError::NotFound(uid))?;
        if !user.groups.insert(group.clone()) {
            return Err(AddGroupError::AlreadyAssigned(group));
        }
        Ok(())
    }

    pub fn remove_group(&mut self, uid: UserId, group: &GroupId) -> Result<(), RemoveGroupError> {
        let user = self
            .users
            .get_mut(&uid)
            .ok_or(RemoveGroupError::NotFound(uid))?;
        if !user.groups.remove(group) {
            return Err(RemoveGroupError::NotAssigned(group.clone()));
        }
        Ok(())
    }

    pub fn update(&mut self, uid: UserId, update: UpdateUser) -> Result<(), UpdateUserError> {
        if let Some(ref name) = update.name
            && self.users.values().any(|u| u.id != uid && u.name == *name)
        {
            return Err(UpdateUserError::DuplicateName(name.clone()));
        }
        let user = self
            .users
            .get_mut(&uid)
            .ok_or(UpdateUserError::NotFound(uid))?;
        if let Some(plain_password) = update.password {
            user.password = password::Password::new(&plain_password)?;
        }
        if let Some(name) = update.name {
            user.name = name;
        }
        if let Some(full_name) = update.full_name {
            user.full_name = full_name;
        }
        Ok(())
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
