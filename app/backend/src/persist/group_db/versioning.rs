use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::model::group::{Group, GroupId};

/// Versioned on-disk format, discriminated by the `"version"` JSON field.
/// New versions are added as variants here; deserialization picks the right one
/// automatically via the tag.
#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
enum StorageEnvelope {
    #[serde(rename = "1")]
    V1 { groups: HashMap<GroupId, Group> },
}

/// Serialization wrapper that always writes the latest envelope format.
/// Borrows the data so no clone is needed in the Drop path.
#[derive(Serialize)]
#[serde(tag = "version")]
pub(super) enum StorageRef<'a> {
    #[serde(rename = "1")]
    V1 { groups: &'a HashMap<GroupId, Group> },
}

impl<'a> StorageRef<'a> {
    pub(super) fn new(groups: &'a HashMap<GroupId, Group>) -> Self {
        Self::V1 { groups }
    }
}

/// Wrapper that handles deserialization from any known on-disk format.
/// Legacy formats are detected and discarded in favor of default groups.
pub(super) struct GroupStorage(pub(super) HashMap<GroupId, Group>);

impl Default for GroupStorage {
    fn default() -> Self {
        Self(super::default::default_groups())
    }
}

impl From<GroupStorage> for HashMap<GroupId, Group> {
    fn from(value: GroupStorage) -> Self {
        value.0
    }
}

impl<'de> Deserialize<'de> for GroupStorage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        // Versioned envelope: has a "version" tag
        if value.get("version").is_some() {
            let envelope: StorageEnvelope =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            return Ok(match envelope {
                StorageEnvelope::V1 { groups } => GroupStorage(groups),
            });
        }

        // Legacy format: discard and use default groups
        if value.is_array() {
            warn!("Discarding legacy group database, replacing with default groups");
            return Ok(Self::default());
        }

        Err(serde::de::Error::custom(
            "unexpected format for group database",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn deserialize_versioned_v1() {
        let json = serde_json::json!({
            "version": "1",
            "groups": {
                "tech.flecs.admin": {
                    "id": "tech.flecs.admin",
                    "name": "Admin",
                    "sub_groups": ["tech.flecs.developer"]
                }
            }
        });
        let storage: GroupStorage = serde_json::from_value(json).unwrap();
        assert_eq!(storage.0.len(), 1);
        assert!(storage.0.contains_key(&GroupId::admin()));
    }

    #[test]
    fn legacy_array_returns_defaults() {
        let json = serde_json::json!([
            {
                "gid": 1,
                "name": "Admin",
                "description": "Administrator group",
                "uids": ["user1", "user2"]
            }
        ]);
        let storage: GroupStorage = serde_json::from_value(json).unwrap();
        let defaults = GroupStorage::default();
        assert_eq!(storage.0.len(), defaults.0.len());
        assert_eq!(
            storage.0.keys().collect::<HashSet<_>>(),
            defaults.0.keys().collect::<HashSet<_>>()
        );
    }

    #[test]
    fn serialize_roundtrip_via_storage_ref() {
        let mut groups = HashMap::new();
        groups.insert(
            GroupId::admin(),
            Group {
                id: GroupId::admin(),
                name: "Admin".to_string(),
                description: None,
                sub_groups: HashSet::from([GroupId::developer()]),
            },
        );

        let storage = StorageRef::new(&groups);
        let json = serde_json::to_value(&storage).unwrap();
        assert_eq!(json["version"], "1");
        assert!(json["groups"]["tech.flecs.admin"].is_object());

        // Deserialize back via GroupStorage
        let wrapper: GroupStorage = serde_json::from_value(json).unwrap();
        assert_eq!(wrapper.0.len(), 1);
    }

    #[test]
    fn unknown_version_fails() {
        let json = serde_json::json!({
            "version": "999",
            "groups": {}
        });
        let result = serde_json::from_value::<GroupStorage>(json);
        assert!(result.is_err());
    }

    #[test]
    fn unexpected_format_fails() {
        let json = serde_json::json!("just a string");
        let result = serde_json::from_value::<GroupStorage>(json);
        assert!(result.is_err());
    }

    #[test]
    fn unversioned_object_fails() {
        let json = serde_json::json!({
            "tech.flecs.admin": {
                "id": "tech.flecs.admin",
                "name": "Admin",
                "sub_groups": []
            }
        });
        let result = serde_json::from_value::<GroupStorage>(json);
        assert!(result.is_err());
    }
}
