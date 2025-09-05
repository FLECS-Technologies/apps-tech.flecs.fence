use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::model::password;

pub type Uid = u16;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct User {
    pub uid: Uid,
    pub name: String,
    pub full_name: String,
    pub password: password::Password,
}
