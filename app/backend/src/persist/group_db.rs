use std::path::PathBuf;

use crate::model::group::Group;

pub struct GroupDB {
    path: PathBuf,
    groups: Vec<Group>,
}

impl GroupDB {
    pub(super) fn new(path: PathBuf) -> anyhow::Result<Self> {
        let groups = super::load_from_file(path.as_path())?;
        Ok(GroupDB { path, groups })
    }
}

impl Drop for GroupDB {
    fn drop(&mut self) {
        super::save_to_file(&self.path, &self.groups)
            .unwrap_or_else(|e| println!("{}", format!("Could not persist group database: {}", e)));
    }
}
