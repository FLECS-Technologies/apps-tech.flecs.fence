pub use id::GroupId;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
mod id;

#[derive(Serialize, Deserialize)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub sub_groups: HashSet<GroupId>,
}
