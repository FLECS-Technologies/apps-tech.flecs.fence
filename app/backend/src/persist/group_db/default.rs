use std::collections::HashMap;

use crate::model::group::{Group, GroupId};

pub(super) fn default_groups() -> HashMap<GroupId, Group> {
    HashMap::new()
}
