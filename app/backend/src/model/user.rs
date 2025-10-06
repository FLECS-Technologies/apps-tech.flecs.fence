use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::model::password;

pub type Uid = u16;

pub const SUPER_ADMIN_ID: u16 = 0;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct User {
    pub uid: Uid,
    pub name: String,
    pub full_name: String,
    pub password: password::Password,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SuperAdmin {
    pub name: String,
    pub full_name: String,
    pub password: String,
}

impl TryFrom<SuperAdmin> for User {
    type Error = anyhow::Error;

    fn try_from(value: SuperAdmin) -> Result<Self, Self::Error> {
        Ok(Self {
            uid: SUPER_ADMIN_ID,
            name: value.name,
            full_name: value.full_name,
            password: password::Password::new(&value.password)?,
        })
    }
}
