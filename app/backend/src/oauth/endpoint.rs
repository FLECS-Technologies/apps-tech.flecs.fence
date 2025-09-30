use oxide_auth::primitives::authorizer::AuthMap;
use oxide_auth::primitives::issuer::TokenMap;
use oxide_auth::primitives::prelude::RandomGenerator;

pub type Authorizer = AuthMap<RandomGenerator>;
pub type Issuer = TokenMap<RandomGenerator>;
