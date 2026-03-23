use std::path::PathBuf;

#[derive(Default)]
pub struct Config {
    pub database: Database,
    pub auth: Auth,
}

pub struct Database {
    pub users_path: PathBuf,
    pub groups_path: PathBuf,
    pub clients_path: PathBuf,
    pub ro_clients_path: PathBuf,
}

pub struct Auth {
    pub issuer_url: url::Url,
    pub casbin_model_path: PathBuf,
    pub casbin_policy_path: PathBuf,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            users_path: "/var/local/lib/fence/users.json".into(),
            groups_path: "/var/local/lib/fence/groups.json".into(),
            clients_path: "/var/local/lib/fence/clients.json".into(),
            ro_clients_path: "/var/local/lib/fence/ro_clients.json".into(),
        }
    }
}

impl Default for Auth {
    fn default() -> Self {
        Self {
            issuer_url: url::Url::parse("http://fence.flecs.local").unwrap(),
            casbin_model_path: "/var/local/lib/fence/casbin_model.conf".into(),
            casbin_policy_path: "/var/local/lib/fence/casbin_policy.csv".into(),
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
}
