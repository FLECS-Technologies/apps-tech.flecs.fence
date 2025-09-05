use std::fmt;

use argon2::{
    Argon2, PasswordVerifier,
    password_hash::{PasswordHash, PasswordHasher, SaltString, rand_core::OsRng},
};
use serde::{
    Deserialize, Serialize,
    de::{self, Visitor},
    ser::Error as _,
};
use std::sync::LazyLock;
use utoipa::ToSchema;
use zeroize::{Zeroize, ZeroizeOnDrop};

static ARGON2: LazyLock<Argon2> = LazyLock::new(|| {
    Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(65_536, 3, 2, None).expect("Argon2 params should be valid"),
    )
});

#[derive(Debug, PartialEq, ToSchema, Zeroize, ZeroizeOnDrop)]
pub enum Password {
    Plain(String),
    Hashed(String),
}

struct PasswordVisitor;

impl Visitor<'_> for PasswordVisitor {
    type Value = Password;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid argon2 phc")
    }

    fn visit_str<E>(self, phc: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let password = Password::from_hash(phc);
        match password {
            Ok(p) => Ok(p),
            _ => Err(E::custom(format!(
                "{} is not a valid argon2 phc string",
                phc
            ))),
        }
    }
}

impl Serialize for Password {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Password::Hashed(s) => serializer.serialize_str(s.as_str()),
            _ => Err(S::Error::custom("Refusing to serialize plaintext password")),
        }
    }
}

impl<'de> Deserialize<'de> for Password {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(PasswordVisitor)
    }
}

impl Password {
    pub fn new(plain: &str) -> anyhow::Result<Self> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = (*ARGON2).hash_password(plain.as_bytes(), &salt)?;
        Ok(Password::Hashed(hash.to_string()))
    }

    pub fn from_hash(phc: &str) -> anyhow::Result<Self> {
        Ok(Password::Hashed(PasswordHash::new(phc)?.to_string()))
    }

    pub fn verify(&self, plain: &str) -> anyhow::Result<()> {
        match self {
            Password::Hashed(s) => {
                Ok((*ARGON2).verify_password(plain.as_bytes(), &PasswordHash::new(s.as_str())?)?)
            }
            _ => Err(anyhow::anyhow!("Attempt to verify plaintext passwords")),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn serialize_deserialize_password_ok() {
        let uut = super::Password::from_hash(
            "$argon2i$v=19$m=65536,t=3,p=2$MDEyMzQ1Njc4OWFiY2RlZg$bTiTXjGTj3v/toFdAb6I3sWoiqFKTvXZ7pyehGPKxN8",
        ).expect("Password should be constructible from hash");

        let ser =
            serde_json::to_string_pretty(&uut).expect("Hashed password should be serializable");

        let de: super::Password =
            serde_json::from_str(ser.as_str()).expect("Hashed password should be deserializable");

        assert_eq!(de, uut);
    }

    #[test]
    fn serialize_password_refused() {
        let uut = super::Password::Plain("Password".to_string());
        serde_json::to_string_pretty(&uut)
            .expect_err("Plaintext password should not be serializable");
    }

    #[test]
    fn deserialize_password_invalid() {
        serde_json::from_str::<super::Password>("password")
            .expect_err("Plaintext password should not be deserializable");
    }
}

#[derive(Serialize, Deserialize)]
pub struct PasswordPolicy {
    len_min: u32,
    len_max: u32,
    need_lower: bool,
    need_upper: bool,
    need_digit: bool,
    need_special: bool,
}

impl PasswordPolicy {
    pub const MIN_LENGTH: u32 = 12;
    pub const MAX_LENGTH: u32 = 63;

    fn new(
        len_min: u32,
        len_max: u32,
        need_lower: bool,
        need_upper: bool,
        need_digit: bool,
        need_special: bool,
    ) -> Result<Self, ()> {
        if len_min > len_max {
            Err(())
        } else {
            Ok(Self {
                len_min,
                len_max,
                need_lower,
                need_upper,
                need_digit,
                need_special,
            })
        }
    }
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            len_min: PasswordPolicy::MIN_LENGTH,
            len_max: PasswordPolicy::MAX_LENGTH,
            need_lower: true,
            need_upper: true,
            need_digit: true,
            need_special: false,
        }
    }
}
