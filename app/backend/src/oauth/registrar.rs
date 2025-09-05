use std::str::FromStr;

use oxide_auth::primitives::registrar::{Client, ClientMap, IgnoreLocalPortUrl, RegisteredUrl};
use oxide_auth::primitives::scope::Scope;
use url::Url;

fn make_flecs_client() -> Client {
    Client::public(
        "flecs",
        RegisteredUrl::IgnorePortOnLocalhost(
            IgnoreLocalPortUrl::new("http://localhost/oauth/callback").unwrap(),
        ),
        Scope::from_str("admin").unwrap(),
    )
}

pub fn build_registrar() -> ClientMap {
    let mut clients = ClientMap::new();
    clients.register_client(make_flecs_client());
    clients
}
