use serde::Serialize;
use user_manager::rest;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

#[derive(Debug, Serialize)]
struct Security;

impl Modify for Security {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(schema) = openapi.components.as_mut() {
            schema.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&Security),
    paths(
        rest::users::get_super_admin,
        rest::users::post_super_admin,
        rest::auth::meta::get_jwk,
        rest::auth::meta::get_issuer,
    ),
    // Top-level security requirement (applies to every operation by default)
    security(
        ("bearerAuth" = [])
    ),
)]
pub struct ApiDoc;
fn main() {
    std::fs::write("./api-spec.yaml", ApiDoc::openapi().to_yaml().unwrap()).unwrap();
}
