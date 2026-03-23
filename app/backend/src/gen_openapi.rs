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
        rest::users::get,
        rest::users::put,
        rest::users::uid::get,
        rest::users::uid::patch,
        rest::users::uid::delete,
        rest::users::self_::patch,
        rest::users::self_::delete,
        rest::users::uid::roles::get,
        rest::users::uid::roles::put,
        rest::users::uid::roles::role::put,
        rest::users::uid::roles::role::delete,
        rest::users::super_admin::get,
        rest::users::super_admin::post,
        rest::meta::jwk::get,
        rest::meta::issuer::get,
        rest::clients::get,
        rest::clients::put,
        rest::clients::cid::get,
        rest::clients::cid::delete,
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
