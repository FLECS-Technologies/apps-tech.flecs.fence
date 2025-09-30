use std::path::PathBuf;

use crate::model::user::User;

pub struct UserDB {
    path: PathBuf,
    users: Vec<User>,
}

impl UserDB {
    pub(super) fn new(path: PathBuf) -> anyhow::Result<Self> {
        let users = super::load_from_file(path.as_path())?;
        Ok(UserDB { path, users })
    }
}

impl Drop for UserDB {
    fn drop(&mut self) {
        super::save_to_file(&self.path, &self.users)
            .unwrap_or_else(|e| println!("{}", format!("Could not persist user database: {}", e)));
    }
}
