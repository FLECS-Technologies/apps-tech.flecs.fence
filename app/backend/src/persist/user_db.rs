use std::path::PathBuf;

use crate::model::user::{SUPER_ADMIN_ID, SuperAdmin, Uid, User};

pub struct UserDB {
    path: PathBuf,
    users: Vec<User>,
}

impl UserDB {
    pub(super) fn new(path: PathBuf) -> anyhow::Result<Self> {
        let users = super::load_from_file(path.as_path())?;
        Ok(UserDB { path, users })
    }

    pub fn query_by_name(&self, name: &str) -> Option<&User> {
        self.users.iter().find(|u| u.name == name)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        super::save_to_file(&self.path, &self.users)
    }

    pub fn query_by_uid(&self, uid: Uid) -> Option<&User> {
        self.users.iter().find(|u| u.uid == uid)
    }

    pub fn contains_super_admin(&self) -> bool {
        self.query_by_uid(SUPER_ADMIN_ID).is_some()
    }

    pub fn set_super_admin(&mut self, super_admin: SuperAdmin) -> anyhow::Result<Option<User>> {
        let super_admin = super_admin.try_into()?;
        if let Some(existing_admin) = self
            .users
            .iter_mut()
            .find(|user| user.uid == SUPER_ADMIN_ID)
        {
            Ok(Some(std::mem::replace(existing_admin, super_admin)))
        } else {
            self.users.push(super_admin);
            Ok(None)
        }
    }
}

impl Drop for UserDB {
    fn drop(&mut self) {
        self.save()
            .unwrap_or_else(|e| println!("Could not persist user database: {e}"));
    }
}
