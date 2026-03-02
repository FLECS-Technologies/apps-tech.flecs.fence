use jsonwebtoken::{Algorithm, EncodingKey};
use oxide_auth::primitives::grant::Grant;
use oxide_auth::primitives::issuer::{IssuedToken, TokenType};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ops::Add;
use thiserror::Error;

const TOKEN_DURATION: chrono::Duration = chrono::Duration::days(1);

#[derive(Debug, Clone, Default)]
pub struct Roles(pub HashSet<String>);

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
    aud: Vec<String>,
    preferred_username: String,
    realm_access: RealmAccess,
    resource_access: ResourceAccess,
}

pub fn issue(
    grant: Grant,
    issuer: url::Url,
    kid: Option<String>,
    encoding_key: &EncodingKey,
) -> Result<IssuedToken, jsonwebtoken::errors::Error> {
    // TODO: Check if grant expires earlier
    let until = chrono::Utc::now().add(TOKEN_DURATION);
    let claims = Claims {
        sub: grant.owner_id,
        exp: until.timestamp() as u64,
        iss: issuer,
        email: "test@flecs.local".to_string(),
        aud: vec!["flecs-core-api".to_string()],
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

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header {
            kid,
            alg: Algorithm::RS256,
            ..jsonwebtoken::Header::default()
        },
        &claims,
        encoding_key,
    )?;
    Ok(IssuedToken {
        token,
        refresh: None,
        until,
        token_type: TokenType::Bearer,
    })
}

#[derive(Debug, Error)]
pub enum VerifyTokenError {
    #[error("No kid in header")]
    NoKid,
    #[error("Key algorithm '{0}' unsupported")]
    UnsupportedAlgorithm(jsonwebtoken::jwk::KeyAlgorithm),
    #[error("No key algorithm present in jwk")]
    NoKeyAlgorithm,
    #[error("Unknown kid '{0}'")]
    UnknownKid(String),
    #[error(transparent)]
    JsonWebToken(#[from] jsonwebtoken::errors::Error),
}

fn algorithm_from_jwk(jwk: &jsonwebtoken::jwk::Jwk) -> Result<Algorithm, VerifyTokenError> {
    Ok(
        match jwk
            .common
            .key_algorithm
            .ok_or(VerifyTokenError::NoKeyAlgorithm)?
        {
            jsonwebtoken::jwk::KeyAlgorithm::HS256 => Algorithm::HS256,
            jsonwebtoken::jwk::KeyAlgorithm::HS384 => Algorithm::HS384,
            jsonwebtoken::jwk::KeyAlgorithm::HS512 => Algorithm::HS512,
            jsonwebtoken::jwk::KeyAlgorithm::ES256 => Algorithm::ES256,
            jsonwebtoken::jwk::KeyAlgorithm::ES384 => Algorithm::ES384,
            jsonwebtoken::jwk::KeyAlgorithm::RS256 => Algorithm::RS256,
            jsonwebtoken::jwk::KeyAlgorithm::RS384 => Algorithm::RS384,
            jsonwebtoken::jwk::KeyAlgorithm::RS512 => Algorithm::RS512,
            jsonwebtoken::jwk::KeyAlgorithm::PS256 => Algorithm::PS256,
            jsonwebtoken::jwk::KeyAlgorithm::PS384 => Algorithm::PS384,
            jsonwebtoken::jwk::KeyAlgorithm::PS512 => Algorithm::PS512,
            jsonwebtoken::jwk::KeyAlgorithm::EdDSA => Algorithm::EdDSA,
            alg => return Err(VerifyTokenError::UnsupportedAlgorithm(alg)),
        },
    )
}

pub fn verify(
    token: &str,
    jwks: &jsonwebtoken::jwk::JwkSet,
    issuer_url: &url::Url,
) -> Result<Roles, VerifyTokenError> {
    let token_header = jsonwebtoken::decode_header(token)?;
    let kid = token_header.kid.as_deref().ok_or(VerifyTokenError::NoKid)?;
    let jwk = jwks
        .find(kid)
        .ok_or_else(|| VerifyTokenError::UnknownKid(kid.to_string()))?;
    let algorithm = algorithm_from_jwk(jwk)?;
    let mut validation = jsonwebtoken::Validation::new(algorithm);
    let decoding_key = jsonwebtoken::DecodingKey::from_jwk(jwk)?;
    validation.set_audience(&["flecs-core-api"]);
    validation.set_issuer(&[issuer_url.as_str()]);
    validation.set_required_spec_claims(&["exp", "aud", "iss", "sub"]);
    let claims = jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation)?.claims;
    Ok(Roles(
        claims
            .realm_access
            .roles
            .into_iter()
            .chain(claims.resource_access.account.roles)
            .collect(),
    ))
}
