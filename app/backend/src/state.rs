use casbin::CoreApi;
use oxide_auth::primitives::prelude::RandomGenerator;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::model::session;
use crate::oauth::endpoint::{Authorizer, Issuer};
use crate::oauth::registrar::{Registrar, build_registrar};
use crate::persist;

#[derive(Clone)]
pub struct AppState {
    pub registrar: Arc<Mutex<Registrar>>,
    pub authorizer: Arc<Mutex<Authorizer>>,
    pub issuer: Arc<Mutex<Issuer>>,
    pub enforcer: Arc<Mutex<casbin::Enforcer>>,
    pub login_sessions: Arc<Mutex<HashSet<session::LoginSession>>>,
    pub user_sessions: Arc<Mutex<HashSet<session::UserSession>>>,
    pub db: Arc<Mutex<persist::Db>>,
}

impl AppState {
    pub fn new(enforcer: casbin::Enforcer) -> Self {
        let db = Arc::new(Mutex::new(persist::Db::default()));
        Self {
            registrar: Arc::new(Mutex::new(build_registrar())),
            authorizer: Arc::new(Mutex::new(Authorizer::new(RandomGenerator::new(16)))),
            issuer: Arc::new(Mutex::new(Issuer::new(db.clone()))),
            enforcer: Arc::new(Mutex::new(enforcer)),
            login_sessions: Arc::new(Mutex::new(HashSet::new())),
            user_sessions: Arc::new(Mutex::new(HashSet::new())),
            db,
        }
    }
}

pub async fn construct_enforcer() -> Result<casbin::Enforcer, casbin::Error> {
    let casbin_model =
        casbin::DefaultModel::from_file("/var/local/lib/fence/casbin_model.conf").await?;
    let casbin_policy = casbin::FileAdapter::new("/var/local/lib/fence/casbin_policy.csv");
    let mut enforcer = casbin::Enforcer::new(casbin_model, casbin_policy).await?;
    enforcer.set_logger(Box::new(casbin::DefaultLogger::default()));
    enforcer.enable_log(true);
    Ok(enforcer)
}
