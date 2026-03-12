use std::{
    borrow::Borrow,
    hash::{Hash, Hasher},
    time::{Duration, Instant},
};

use uuid::Uuid;

use crate::model::user::UserId;

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

    pub fn is_expired(&self) -> bool {
        self.expire_at < Instant::now()
    }
}

#[derive(Eq)]
pub struct UserSession {
    sid: String,
    uid: UserId,
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
    pub fn new(uid: UserId) -> Self {
        Self {
            sid: new_sid(),
            uid,
        }
    }

    pub fn get_sid(&self) -> &str {
        &self.sid
    }

    pub fn get_uid(&self) -> UserId {
        self.uid
    }
}

impl Default for UserSession {
    fn default() -> Self {
        Self::new(0)
    }
}

fn new_sid() -> String {
    Uuid::new_v4().as_simple().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_session_with_future_expiry_is_not_expired() {
        let session = LoginSession {
            sid: new_sid(),
            q: "q=test".into(),
            expire_at: Instant::now() + Duration::from_secs(60),
        };
        assert!(!session.is_expired());
    }

    #[test]
    fn login_session_with_past_expiry_is_expired() {
        let session = LoginSession {
            sid: new_sid(),
            q: "q=test".into(),
            expire_at: Instant::now() - Duration::from_secs(1),
        };
        assert!(session.is_expired());
    }
}
