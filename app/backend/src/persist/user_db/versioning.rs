use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::model::{
    group::GroupId,
    password::Password,
    user::{SUPER_ADMIN_ID, User, UserId},
};

#[derive(Deserialize)]
#[serde(tag = "version")]
enum StorageEnvelope {
    #[serde(rename = "1")]
    V1 { users: Vec<User> },
}

#[derive(Serialize)]
#[serde(tag = "version")]
pub(super) enum StorageRef<'a> {
    #[serde(rename = "1")]
    V1 { users: Vec<&'a User> },
}

impl<'a> StorageRef<'a> {
    pub(super) fn new(users: &'a HashMap<UserId, User>) -> Self {
        Self::V1 {
            users: users.values().collect(),
        }
    }
}

/// Legacy user format: `uid` instead of `id`, no `groups` field.
#[derive(Deserialize)]
struct LegacyUser {
    uid: UserId,
    name: String,
    full_name: String,
    password: Password,
}

impl LegacyUser {
    fn migrate(self) -> User {
        let groups = if self.uid == SUPER_ADMIN_ID {
            warn!(
                "Legacy user '{}' (uid={}) is super admin, assigning to admin group",
                self.name, self.uid
            );
            [GroupId::admin()].into()
        } else {
            Default::default()
        };
        User {
            id: self.uid,
            name: self.name,
            full_name: self.full_name,
            password: self.password,
            groups,
        }
    }
}

#[derive(Default)]
pub(super) struct UserStorage(pub(super) HashMap<UserId, User>);

impl From<UserStorage> for HashMap<UserId, User> {
    fn from(value: UserStorage) -> Self {
        value.0
    }
}

fn vec_to_map(users: Vec<User>) -> HashMap<UserId, User> {
    users.into_iter().map(|u| (u.id, u)).collect()
}

impl<'de> Deserialize<'de> for UserStorage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        if value.get("version").is_some() {
            let envelope: StorageEnvelope =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            return Ok(match envelope {
                StorageEnvelope::V1 { users } => UserStorage(vec_to_map(users)),
            });
        }

        if value.is_array() {
            let legacy: Vec<LegacyUser> =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            warn!(
                "Migrating legacy user database ({} users) to versioned format",
                legacy.len()
            );
            let users = legacy
                .into_iter()
                .map(|u| {
                    let id = u.uid;
                    (id, u.migrate())
                })
                .collect();
            return Ok(UserStorage(users));
        }

        Err(serde::de::Error::custom(
            "unexpected format for user database",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn test_password() -> Password {
        Password::new("TestPassword123!").unwrap()
    }

    #[test]
    fn deserialize_versioned_v1() {
        let password = test_password();
        let json = serde_json::json!({
            "version": "1",
            "users": [
                {
                    "id": 0,
                    "name": "admin",
                    "full_name": "Admin",
                    "password": serde_json::to_value(&password).unwrap(),
                    "groups": ["tech.flecs.admin"]
                }
            ]
        });
        let storage: UserStorage = serde_json::from_value(json).unwrap();
        assert_eq!(storage.0.len(), 1);
        assert!(storage.0.contains_key(&0));
    }

    #[test]
    fn legacy_super_admin_gets_admin_group() {
        let password = test_password();
        let json = serde_json::json!([
            {
                "uid": 0,
                "name": "admin",
                "full_name": "Super Admin",
                "password": serde_json::to_value(&password).unwrap()
            }
        ]);
        let storage: UserStorage = serde_json::from_value(json).unwrap();
        assert_eq!(storage.0.len(), 1);
        let user = storage.0.get(&0).unwrap();
        assert!(user.groups.contains(&GroupId::admin()));
    }

    #[test]
    fn legacy_non_admin_gets_no_groups() {
        let password = test_password();
        let json = serde_json::json!([
            {
                "uid": 1,
                "name": "regular",
                "full_name": "Regular User",
                "password": serde_json::to_value(&password).unwrap()
            }
        ]);
        let storage: UserStorage = serde_json::from_value(json).unwrap();
        let user = storage.0.get(&1).unwrap();
        assert!(user.groups.is_empty());
    }

    #[test]
    fn serialize_roundtrip_via_storage_ref() {
        let mut users = HashMap::new();
        users.insert(
            0,
            User {
                id: 0,
                name: "admin".to_string(),
                full_name: "Admin".to_string(),
                password: test_password(),
                groups: HashSet::from([GroupId::admin()]),
            },
        );

        let storage = StorageRef::new(&users);
        let json = serde_json::to_value(&storage).unwrap();
        assert_eq!(json["version"], "1");
        assert!(json["users"].is_array());

        let wrapper: UserStorage = serde_json::from_value(json).unwrap();
        assert_eq!(wrapper.0.len(), 1);
        assert!(wrapper.0.contains_key(&0));
    }

    #[test]
    fn unknown_version_fails() {
        let json = serde_json::json!({
            "version": "999",
            "users": []
        });
        let result = serde_json::from_value::<UserStorage>(json);
        assert!(result.is_err());
    }

    #[test]
    fn unexpected_format_fails() {
        let json = serde_json::json!("just a string");
        let result = serde_json::from_value::<UserStorage>(json);
        assert!(result.is_err());
    }
}
