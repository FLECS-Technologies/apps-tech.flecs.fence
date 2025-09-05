use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Group {
    gid: u16,
    name: String,
    description: String,
    uids: Vec<String>,
}
