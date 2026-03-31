mod common;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use http::Request;
use std::ops::Add;

fn json_body(json: &str) -> axum::body::Body {
    axum::body::Body::from(json.to_string())
}

fn form_body(params: &str) -> axum::body::Body {
    axum::body::Body::from(params.to_string())
}

const VALID_PASSWORD: &str = "TestPassword123";

fn super_admin_json() -> String {
    format!(r#"{{"name": "admin", "full_name": "Super Admin", "password": "{VALID_PASSWORD}"}}"#)
}

async fn setup_admin(app: &common::TestApp) -> String {
    let req = Request::post("/users/super-admin")
        .header("content-type", "application/json")
        .body(json_body(&super_admin_json()))
        .unwrap();
    app.request(req).await;
    app.mint_token(0)
}

/// Create a client with secret auth and return (client_id, client_secret).
async fn create_secret_client(app: &common::TestApp, token: &str, name: &str) -> (String, String) {
    let req = Request::post("/clients")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(&format!(
            r#"{{"name": "{name}", "auth_method": {{"type": "Secret"}}, "groups": ["tech.flecs.admin"]}}"#
        )))
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CREATED, "body: {body}");
    let resp: serde_json::Value = serde_json::from_str(&body).unwrap();
    let id = resp["id"].as_str().unwrap().to_string();
    let secret = resp["secret"].as_str().unwrap().to_string();
    (id, secret)
}

#[tokio::test]
async fn test_client_credentials_with_body_params() {
    let app = common::TestApp::new().await;
    let token = setup_admin(&app).await;
    let (client_id, client_secret) = create_secret_client(&app, &token, "body-svc").await;

    let form = format!(
        "grant_type=client_credentials&client_id={client_id}&client_secret={client_secret}"
    );
    let req = Request::post("/oauth/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(form_body(&form))
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::OK, "body: {body}");

    let resp: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(resp["access_token"].is_string());
    assert_eq!(resp["token_type"], "Bearer");
    assert_eq!(resp["expires_in"], 600);
}

#[tokio::test]
async fn test_client_credentials_with_basic_auth() {
    let app = common::TestApp::new().await;
    let token = setup_admin(&app).await;
    let (client_id, client_secret) = create_secret_client(&app, &token, "basic-svc").await;

    let credentials = STANDARD.encode(format!("{client_id}:{client_secret}"));
    let req = Request::post("/oauth/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .header("authorization", format!("Basic {credentials}"))
        .body(form_body("grant_type=client_credentials"))
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::OK, "body: {body}");

    let resp: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(resp["access_token"].is_string());
    assert_eq!(resp["token_type"], "Bearer");
    assert_eq!(resp["expires_in"], 600);
}

#[tokio::test]
async fn test_client_credentials_wrong_secret() {
    let app = common::TestApp::new().await;
    let token = setup_admin(&app).await;
    let (client_id, _) = create_secret_client(&app, &token, "wrong-secret-svc").await;

    let form = format!("grant_type=client_credentials&client_id={client_id}&client_secret=wrong");
    let req = Request::post("/oauth/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(form_body(&form))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_client_credentials_unknown_client() {
    let app = common::TestApp::new().await;

    let form = "grant_type=client_credentials&client_id=00000000-0000-0000-0000-000000000000&client_secret=nope";
    let req = Request::post("/oauth/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(form_body(form))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_client_credentials_missing_client_id() {
    let app = common::TestApp::new().await;

    let req = Request::post("/oauth/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(form_body("grant_type=client_credentials&client_secret=foo"))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_client_credentials_token_has_correct_claims() {
    let app = common::TestApp::new().await;
    let token = setup_admin(&app).await;
    let (client_id, client_secret) = create_secret_client(&app, &token, "claims-svc").await;

    let form = format!(
        "grant_type=client_credentials&client_id={client_id}&client_secret={client_secret}"
    );
    let req = Request::post("/oauth/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(form_body(&form))
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::OK, "body: {body}");

    let resp: serde_json::Value = serde_json::from_str(&body).unwrap();
    let access_token = resp["access_token"].as_str().unwrap();

    // Decode without verification to inspect claims
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.insecure_disable_signature_validation();
    validation.set_audience(&["flecs-core-api"]);
    validation.validate_exp = false;
    let token_data = jsonwebtoken::decode::<serde_json::Value>(
        access_token,
        &jsonwebtoken::DecodingKey::from_secret(&[]),
        &validation,
    )
    .unwrap();

    let claims = token_data.claims;
    assert_eq!(claims["sub"], client_id);
    assert_eq!(claims["token_type"], "client");
    assert!(claims.get("email").is_none());
    assert!(claims.get("preferred_username").is_none());
    let aud = claims["aud"].as_array().unwrap();
    assert!(aud.iter().any(|a| a == "flecs-core-api"));
    assert!(aud.iter().any(|a| a == "fence-api"));
}

/// Create a client with Fence-generated certificate and return (client_id, private_key_pem).
async fn create_cert_client(app: &common::TestApp, token: &str, name: &str) -> (String, String) {
    let req = Request::post("/clients")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(&format!(
            r#"{{"name": "{name}", "auth_method": {{"type": "Certificate"}}, "groups": ["tech.flecs.admin"]}}"#
        )))
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CREATED, "body: {body}");
    let resp: serde_json::Value = serde_json::from_str(&body).unwrap();
    let id = resp["id"].as_str().unwrap().to_string();
    let key = resp["private_key"].as_str().unwrap().to_string();
    (id, key)
}

/// Sign a JWT assertion for client_credentials certificate auth.
fn sign_assertion(client_id: &str, private_key_pem: &str) -> String {
    let key = jsonwebtoken::EncodingKey::from_rsa_pem(private_key_pem.as_bytes()).unwrap();
    let now = chrono::Utc::now();
    let claims = serde_json::json!({
        "iss": client_id,
        "sub": client_id,
        "aud": ["flecs-core-api", "fence-api"],
        "exp": now.add(chrono::Duration::minutes(5)).timestamp(),
        "iat": now.timestamp(),
    });
    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
        &claims,
        &key,
    )
    .unwrap()
}

#[tokio::test]
async fn test_client_credentials_certificate_flow() {
    let app = common::TestApp::new().await;
    let token = setup_admin(&app).await;
    let (client_id, private_key) = create_cert_client(&app, &token, "cert-svc").await;

    let assertion = sign_assertion(&client_id, &private_key);
    let form = format!(
        "grant_type=client_credentials&client_id={client_id}&client_assertion_type=urn%3Aietf%3Aparams%3Aoauth%3Aclient-assertion-type%3Ajwt-bearer&client_assertion={assertion}"
    );
    let req = Request::post("/oauth/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(form_body(&form))
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::OK, "body: {body}");

    let resp: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(resp["access_token"].is_string());
    assert_eq!(resp["token_type"], "Bearer");
    assert_eq!(resp["expires_in"], 600);
}

#[tokio::test]
async fn test_client_credentials_certificate_wrong_key() {
    let app = common::TestApp::new().await;
    let token = setup_admin(&app).await;
    let (client_id, _) = create_cert_client(&app, &token, "cert-wrong-key").await;

    // Generate a different key to sign with
    let wrong_key = openssl::rsa::Rsa::generate(2048).unwrap();
    let wrong_pem = String::from_utf8(
        openssl::pkey::PKey::from_rsa(wrong_key)
            .unwrap()
            .private_key_to_pem_pkcs8()
            .unwrap(),
    )
    .unwrap();
    let assertion = sign_assertion(&client_id, &wrong_pem);

    let form = format!(
        "grant_type=client_credentials&client_id={client_id}&client_assertion_type=urn%3Aietf%3Aparams%3Aoauth%3Aclient-assertion-type%3Ajwt-bearer&client_assertion={assertion}"
    );
    let req = Request::post("/oauth/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(form_body(&form))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_client_credentials_certificate_wrong_sub() {
    let app = common::TestApp::new().await;
    let token = setup_admin(&app).await;
    let (client_id, private_key) = create_cert_client(&app, &token, "cert-wrong-sub").await;

    // Sign assertion with wrong sub
    let key = jsonwebtoken::EncodingKey::from_rsa_pem(private_key.as_bytes()).unwrap();
    let now = chrono::Utc::now();
    let claims = serde_json::json!({
        "iss": client_id,
        "sub": "00000000-0000-0000-0000-000000000000",
        "aud": ["flecs-core-api", "fence-api"],
        "exp": now.add(chrono::Duration::minutes(5)).timestamp(),
        "iat": now.timestamp(),
    });
    let assertion = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
        &claims,
        &key,
    )
    .unwrap();

    let form = format!(
        "grant_type=client_credentials&client_id={client_id}&client_assertion_type=urn%3Aietf%3Aparams%3Aoauth%3Aclient-assertion-type%3Ajwt-bearer&client_assertion={assertion}"
    );
    let req = Request::post("/oauth/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(form_body(&form))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_client_credentials_certificate_missing_assertion() {
    let app = common::TestApp::new().await;
    let token = setup_admin(&app).await;
    let (client_id, _) = create_cert_client(&app, &token, "cert-no-assertion").await;

    let form = format!(
        "grant_type=client_credentials&client_id={client_id}&client_assertion_type=urn%3Aietf%3Aparams%3Aoauth%3Aclient-assertion-type%3Ajwt-bearer"
    );
    let req = Request::post("/oauth/token")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(form_body(&form))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::BAD_REQUEST);
}
