use std::sync::{Arc, Mutex};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as b64url;
use jsonwebtoken::jwk::{
    AlgorithmParameters, CommonParameters, KeyAlgorithm, PublicKeyUse, RSAKeyParameters, RSAKeyType,
};
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use oxide_auth::primitives::authorizer::AuthMap;
use oxide_auth::primitives::grant::Grant;
use oxide_auth::primitives::issuer::{IssuedToken, RefreshedToken};
use oxide_auth::primitives::prelude::RandomGenerator;
use tracing::{debug, error, warn};

use crate::persist;

pub type Authorizer = AuthMap<RandomGenerator>;

pub struct Issuer {
    pub jwk: jsonwebtoken::jwk::Jwk,
    pub encoding_key: jsonwebtoken::EncodingKey,
    pub url: url::Url,
    db: Arc<Mutex<persist::Db>>,
}

impl Issuer {
    pub fn new(db: Arc<Mutex<persist::Db>>, issuer_url: url::Url) -> Self {
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
                key_id: Some(uuid::Uuid::new_v4().to_string()),
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
            url: issuer_url,
            jwk,
            encoding_key,
            db,
        }
    }
}

impl oxide_auth::primitives::issuer::Issuer for Issuer {
    fn issue(&mut self, grant: Grant) -> Result<IssuedToken, ()> {
        match crate::token::issue(
            grant,
            self.url.clone(),
            self.jwk.common.key_id.clone(),
            &self.encoding_key,
            self.db.clone(),
        ) {
            Ok(token) => {
                debug!("Created token");
                Ok(token)
            }
            Err(e) => {
                error!("Error encoding token: {e}");
                Err(())
            }
        }
    }

    fn refresh(&mut self, _refresh: &str, _grant: Grant) -> Result<RefreshedToken, ()> {
        warn!("fn refresh unimplemented");
        Err(())
    }

    fn recover_token<'a>(&'a self, _: &'a str) -> Result<Option<Grant>, ()> {
        warn!("fn recover_token unimplemented");
        Err(())
    }

    fn recover_refresh<'a>(&'a self, _: &'a str) -> Result<Option<Grant>, ()> {
        warn!("fn recover_refresh unimplemented");
        Err(())
    }
}
