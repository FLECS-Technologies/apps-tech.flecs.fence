use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug, ToSchema)]
pub struct GroupId(String);

impl AsRef<str> for GroupId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl GroupId {
    pub fn admin() -> Self {
        Self("tech.flecs.admin".to_string())
    }

    pub fn developer() -> Self {
        Self("tech.flecs.developer".to_string())
    }

    pub fn technician() -> Self {
        Self("tech.flecs.technician".to_string())
    }

    pub fn operator() -> Self {
        Self("tech.flecs.operator".to_string())
    }
}

impl From<String> for GroupId {
    fn from(s: String) -> Self {
        Self(s)
    }
}
