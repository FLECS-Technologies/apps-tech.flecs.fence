use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as b64url;
use jsonwebtoken::Algorithm;
use jsonwebtoken::jwk::{
    AlgorithmParameters, CommonParameters, KeyAlgorithm, PublicKeyUse, RSAKeyParameters, RSAKeyType,
};
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use oxide_auth::primitives::authorizer::AuthMap;
use oxide_auth::primitives::grant::Grant;
use oxide_auth::primitives::issuer::{IssuedToken, RefreshedToken, TokenType};
use oxide_auth::primitives::prelude::RandomGenerator;
use serde::{Deserialize, Serialize};
use std::ops::Add;

pub type Authorizer = AuthMap<RandomGenerator>;

pub struct Issuer {
    pub jwk: jsonwebtoken::jwk::Jwk,
    pub encoding_key: jsonwebtoken::EncodingKey,
    pub url: url::Url,
}

impl Issuer {
    pub fn new() -> Self {
        let rsa = Rsa::generate(2048).unwrap();
        let pkey = PKey::from_rsa(rsa).unwrap();
        let private_key: Vec<u8> = pkey.private_key_to_pem_pkcs8().unwrap();
        let encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(&private_key).unwrap();
        let n = pkey.rsa().unwrap().n().to_vec();
        let n = b64url.encode(&n);
        let e = pkey.rsa().unwrap().e().to_vec();
        let e = b64url.encode(&e);
        let jwk = jsonwebtoken::jwk::Jwk {
            common: CommonParameters {
                key_id: Some("flecs-kid".to_string()),
                public_key_use: Some(PublicKeyUse::Signature),
                key_algorithm: Some(KeyAlgorithm::RS256),
                ..Default::default()
            },
            algorithm: AlgorithmParameters::RSA(RSAKeyParameters {
                n,
                e,
                key_type: RSAKeyType::RSA,
            }),
        };
        Self {
            url: url::Url::parse("http://fence.flecs.local").unwrap(),
            jwk,
            encoding_key,
        }
    }
}

impl oxide_auth::primitives::issuer::Issuer for Issuer {
    fn issue(&mut self, grant: Grant) -> Result<IssuedToken, ()> {
        #[derive(Debug, Serialize, Deserialize)]
        struct RealmAccess {
            roles: Vec<String>,
        }
        type Account = RealmAccess;
        #[derive(Debug, Serialize, Deserialize)]
        struct ResourceAccess {
            account: Account,
        }
        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            sub: String,
            exp: u64,
            iss: url::Url,
            email: String,
            preferred_username: String,
            realm_access: RealmAccess,
            resource_access: ResourceAccess,
        }
        // TODO: Check if grant expires earlier
        let until = chrono::Utc::now().add(chrono::Duration::hours(1));
        let claims = Claims {
            sub: grant.owner_id,
            exp: until.timestamp() as u64,
            iss: self.url.clone(),
            email: "test@flecs.local".to_string(),
            preferred_username: "Super Admin".to_string(),
            realm_access: RealmAccess {
                roles: vec!["tech.flecs.core.admin".to_string()],
            },
            resource_access: ResourceAccess {
                account: Account {
                    roles: vec!["tech.flecs.core.admin".to_string()],
                },
            },
        };

        match jsonwebtoken::encode(
            &jsonwebtoken::Header {
                kid: Some("flecs-kid".to_string()),
                alg: Algorithm::RS256,
                ..jsonwebtoken::Header::default()
            },
            &claims,
            &self.encoding_key,
        ) {
            Ok(token) => {
                println!("Created token");
                Ok(IssuedToken {
                    token,
                    refresh: None,
                    until,
                    token_type: TokenType::Bearer,
                })
            }
            Err(e) => {
                eprintln!("Error encoding token: {e}");
                Err(())
            }
        }
    }

    fn refresh(&mut self, _refresh: &str, _grant: Grant) -> Result<RefreshedToken, ()> {
        eprintln!("fn refresh unimplemented");
        Err(())
    }

    fn recover_token<'a>(&'a self, _: &'a str) -> Result<Option<Grant>, ()> {
        eprintln!("fn recover_token unimplemented");
        Err(())
    }

    fn recover_refresh<'a>(&'a self, _: &'a str) -> Result<Option<Grant>, ()> {
        eprintln!("fn recover_refresh unimplemented");
        Err(())
    }
}
