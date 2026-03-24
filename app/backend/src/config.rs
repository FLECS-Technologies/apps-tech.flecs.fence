use std::path::PathBuf;

use serde::Deserialize;

#[derive(Default)]
pub struct Config {
    pub database: Database,
    pub auth: Auth,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        Ok(Self {
            database: envy::prefixed("FENCE_DATABASE_").from_env()?,
            auth: envy::prefixed("FENCE_AUTH_").from_env()?,
        })
    }
}

fn default_users_path() -> PathBuf {
    "/var/local/lib/fence/users.json".into()
}

fn default_groups_path() -> PathBuf {
    "/var/local/lib/fence/groups.json".into()
}

fn default_clients_path() -> PathBuf {
    "/var/local/lib/fence/clients.json".into()
}

fn default_ro_clients_path() -> PathBuf {
    "/var/local/lib/fence/ro_clients.json".into()
}

fn default_issuer_url() -> url::Url {
    url::Url::parse("http://fence.flecs.local").unwrap()
}

fn default_casbin_model_path() -> PathBuf {
    "/var/local/lib/fence/casbin_model.conf".into()
}

fn default_casbin_policy_path() -> PathBuf {
    "/var/local/lib/fence/casbin_policy.csv".into()
}

#[derive(Deserialize)]
pub struct Database {
    #[serde(default = "default_users_path")]
    pub users_path: PathBuf,
    #[serde(default = "default_groups_path")]
    pub groups_path: PathBuf,
    #[serde(default = "default_clients_path")]
    pub clients_path: PathBuf,
    #[serde(default = "default_ro_clients_path")]
    pub ro_clients_path: PathBuf,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            users_path: default_users_path(),
            groups_path: default_groups_path(),
            clients_path: default_clients_path(),
            ro_clients_path: default_ro_clients_path(),
        }
    }
}

#[derive(Deserialize)]
pub struct Auth {
    #[serde(default = "default_issuer_url")]
    pub issuer_url: url::Url,
    #[serde(default = "default_casbin_model_path")]
    pub casbin_model_path: PathBuf,
    #[serde(default = "default_casbin_policy_path")]
    pub casbin_policy_path: PathBuf,
}

impl Default for Auth {
    fn default() -> Self {
        Self {
            issuer_url: default_issuer_url(),
            casbin_model_path: default_casbin_model_path(),
            casbin_policy_path: default_casbin_policy_path(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_auth_config() {
        let _auth = Auth::default();
    }

    #[test]
    fn from_env_uses_defaults_when_no_vars_set() {
        let config = Config::from_env().unwrap();
        let default = Config::default();
        assert_eq!(config.database.users_path, default.database.users_path);
        assert_eq!(config.auth.issuer_url, default.auth.issuer_url);
    }
}
