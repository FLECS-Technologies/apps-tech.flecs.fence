use oxide_auth::frontends::simple::endpoint::Generic;
use oxide_auth::primitives::authorizer::AuthMap;
use oxide_auth::primitives::issuer::TokenMap;
use oxide_auth::primitives::prelude::RandomGenerator;
use oxide_auth::primitives::registrar::ClientMap;

pub type Authorizer = AuthMap<RandomGenerator>;
pub type Issuer = TokenMap<RandomGenerator>;

pub type Endpoint = Generic<ClientMap, AuthMap<RandomGenerator>, TokenMap<RandomGenerator>>;
