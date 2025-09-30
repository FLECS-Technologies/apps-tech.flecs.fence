use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use oxide_auth::primitives::prelude::RandomGenerator;

use crate::model::session;
use crate::oauth::endpoint::{Authorizer, Issuer};
use crate::oauth::registrar::{Registrar, build_registrar};
use crate::persist;

#[derive(Clone)]
pub struct AppState {
    pub registrar: Arc<Mutex<Registrar>>,
    pub authorizer: Arc<Mutex<Authorizer>>,
    pub issuer: Arc<Mutex<Issuer>>,
    pub login_sessions: Arc<Mutex<HashSet<session::LoginSession>>>,
    pub user_sessions: Arc<Mutex<HashSet<session::UserSession>>>,
    pub db: Arc<Mutex<persist::Db>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            registrar: Arc::new(Mutex::new(build_registrar())),
            authorizer: Arc::new(Mutex::new(Authorizer::new(RandomGenerator::new(16)))),
            issuer: Arc::new(Mutex::new(Issuer::new(RandomGenerator::new(16)))),
            login_sessions: Arc::new(Mutex::new(HashSet::new())),
            user_sessions: Arc::new(Mutex::new(HashSet::new())),
            db: Arc::new(Mutex::new(persist::Db::default())),
        }
    }
}
