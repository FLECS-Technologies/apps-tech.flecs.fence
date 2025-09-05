use std::{
    borrow::Borrow,
    hash::{Hash, Hasher},
    time::{Duration, Instant},
};

use uuid::Uuid;

use crate::model::user::Uid;

const LOGIN_SESSION_EXPIRY: Duration = Duration::from_secs(5 * 60);

#[derive(Eq)]
pub struct LoginSession {
    sid: String,
    q: String,
    expire_at: Instant,
}

impl PartialEq for LoginSession {
    fn eq(&self, other: &Self) -> bool {
        self.sid.eq(&other.sid)
    }
}

impl Hash for LoginSession {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sid.hash(state);
    }
}

impl Borrow<str> for LoginSession {
    fn borrow(&self) -> &str {
        &self.sid
    }
}

impl LoginSession {
    pub fn new(q: String) -> Self {
        Self {
            sid: new_sid(),
            q,
            expire_at: Instant::now() + LOGIN_SESSION_EXPIRY,
        }
    }

    pub fn get_sid(&self) -> &str {
        &self.sid
    }

    pub fn get_q(&self) -> &str {
        &self.q
    }
}

#[derive(Eq)]
pub struct UserSession {
    sid: String,
    uid: Uid,
}

impl PartialEq for UserSession {
    fn eq(&self, other: &Self) -> bool {
        self.sid.eq(&other.sid)
    }
}

impl Hash for UserSession {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sid.hash(state);
    }
}

impl Borrow<str> for UserSession {
    fn borrow(&self) -> &str {
        &self.sid
    }
}

impl UserSession {
    pub fn new() -> Self {
        Self {
            sid: new_sid(),
            uid: Uid::default(),
        }
    }

    pub fn get_sid(&self) -> &str {
        &self.sid
    }

    pub fn get_uid(&self) -> Uid {
        self.uid
    }
}

impl Default for UserSession {
    fn default() -> Self {
        Self::new()
    }
}

fn new_sid() -> String {
    Uuid::new_v4().as_simple().to_string()
}
