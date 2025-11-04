use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::LazyLock;

use oxide_auth::endpoint::PreGrant;
use oxide_auth::primitives::prelude::ClientUrl;
use oxide_auth::primitives::registrar::{
    Argon2, BoundClient, Client, EncodedClient, ExactUrl, PasswordPolicy, RegisteredClient,
    RegisteredUrl, RegistrarError,
};
use oxide_auth::primitives::scope::Scope;

fn make_flecs_client() -> Client {
    Client::public(
        "flecs",
        RegisteredUrl::Exact(ExactUrl::new("https://*/".into()).unwrap()),
        Scope::from_str("admin").unwrap(),
    )
}

pub fn build_registrar() -> Registrar {
    let mut clients = Registrar::new();
    clients.register_client(make_flecs_client());
    clients
}

#[derive(Default)]
pub struct Registrar {
    clients: HashMap<String, EncodedClient>,
    password_policy: Option<Box<dyn PasswordPolicy>>,
}

static DEFAULT_PASSWORD_POLICY: LazyLock<Argon2> = LazyLock::new(Argon2::default);

impl Registrar {
    pub fn new() -> Self {
        Registrar::default()
    }

    pub fn register_client(&mut self, client: Client) {
        let password_policy = Self::current_policy(&self.password_policy);
        let encoded_client = client.encode(password_policy);
        self.clients
            .insert(encoded_client.client_id.clone(), encoded_client);
    }

    pub fn set_password_policy<P: PasswordPolicy + 'static>(&mut self, new_policy: P) {
        self.password_policy = Some(Box::new(new_policy))
    }

    fn current_policy(policy: &Option<Box<dyn PasswordPolicy>>) -> &dyn PasswordPolicy {
        policy
            .as_ref()
            .map(|boxed| &**boxed)
            .unwrap_or(&*DEFAULT_PASSWORD_POLICY)
    }
}

impl oxide_auth::primitives::registrar::Registrar for Registrar {
    fn bound_redirect<'a>(&self, bound: ClientUrl<'a>) -> Result<BoundClient<'a>, RegistrarError> {
        let client = match self.clients.get(bound.client_id.as_ref()) {
            None => return Err(RegistrarError::Unspecified),
            Some(stored) => stored,
        };

        // URI matching is modified so that 'https://*/' means "open redirect" to any
        // location the user-agent asks for
        fn is_wildcard_redirect(u: &RegisteredUrl) -> bool {
            match u {
                RegisteredUrl::Exact(url) => url.as_str() == "https://*/",
                _ => false,
            }
        }

        let registered_url = if is_wildcard_redirect(&client.redirect_uri) {
            match bound.redirect_uri {
                None => return Err(RegistrarError::Unspecified),
                Some(uri) => RegisteredUrl::Exact((*uri).clone()),
            }
        } else {
            match bound.redirect_uri {
                None => client.redirect_uri.clone(),
                Some(url) => {
                    let original = std::iter::once(&client.redirect_uri);
                    let alternatives = client.additional_redirect_uris.iter();
                    if original
                        .chain(alternatives)
                        .any(|registered| *registered == *url.as_ref())
                    {
                        RegisteredUrl::Exact((*url).clone())
                    } else {
                        return Err(RegistrarError::Unspecified);
                    }
                }
            }
        };

        Ok(BoundClient {
            client_id: bound.client_id,
            redirect_uri: Cow::Owned(registered_url),
        })
    }

    fn negotiate(
        &self,
        bound: BoundClient,
        _scope: Option<Scope>,
    ) -> Result<PreGrant, RegistrarError> {
        let client = self
            .clients
            .get(bound.client_id.as_ref())
            .expect("Bound client appears to not have been constructed with this registrar");
        Ok(PreGrant {
            client_id: bound.client_id.into_owned(),
            redirect_uri: bound.redirect_uri.into_owned(),
            scope: client.default_scope.clone(),
        })
    }

    fn check(&self, client_id: &str, passphrase: Option<&[u8]>) -> Result<(), RegistrarError> {
        let password_policy = Self::current_policy(&self.password_policy);

        self.clients
            .get(client_id)
            .ok_or(RegistrarError::Unspecified)
            .and_then(|client| {
                RegisteredClient::new(client, password_policy).check_authentication(passphrase)
            })?;

        Ok(())
    }
}
