use jsonwebtoken::{Algorithm, EncodingKey};
use oxide_auth::primitives::grant::Grant;
use oxide_auth::primitives::issuer::{IssuedToken, TokenType};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ops::Add;

const TOKEN_DURATION: chrono::Duration = chrono::Duration::days(1);

#[derive(Clone, Default)]
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