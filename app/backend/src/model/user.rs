use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::model::{group::GroupId, password};

pub type UserId = u16;

pub const SUPER_ADMIN_ID: u16 = 0;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub full_name: String,
    pub password: password::Password,
    pub groups: HashSet<GroupId>,
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
            id: SUPER_ADMIN_ID,
            name: value.name,
            full_name: value.full_name,
            password: password::Password::new(&value.password)?,
            groups: [GroupId::admin()].into(),
        })
    }
}
