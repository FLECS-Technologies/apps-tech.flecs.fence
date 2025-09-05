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
