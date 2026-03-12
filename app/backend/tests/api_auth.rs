mod common;

use http::Request;

const VALID_PASSWORD: &str = "TestPassword123";

fn super_admin_json() -> String {
    format!(r#"{{"name": "admin", "full_name": "Super Admin", "password": "{VALID_PASSWORD}"}}"#)
}

fn json_body(json: &str) -> axum::body::Body {
    axum::body::Body::from(json.to_string())
}

#[tokio::test]
async fn test_get_login_returns_html() {
    let app = common::TestApp::new().await;
    let req = Request::get("/login")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::OK);
    assert!(body.contains("<form"), "Expected HTML login form");
}

#[tokio::test]
async fn test_post_login_unknown_user() {
    let app = common::TestApp::new().await;
    let req = Request::post("/login")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(axum::body::Body::from("username=nobody&password=wrong"))
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::FORBIDDEN);
    assert!(body.contains("Invalid username and/or password"));
}

#[tokio::test]
async fn test_post_login_wrong_password() {
    let app = common::TestApp::new().await;

    // Create super admin
    let req = Request::post("/users/super-admin")
        .header("content-type", "application/json")
        .body(json_body(&super_admin_json()))
        .unwrap();
    app.request(req).await;

    let req = Request::post("/login")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(axum::body::Body::from(
            "username=admin&password=WrongPassword",
        ))
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::FORBIDDEN);
    assert!(body.contains("Invalid username and/or password"));
}

#[tokio::test]
async fn test_post_login_success() {
    let app = common::TestApp::new().await;

    // Create super admin
    let req = Request::post("/users/super-admin")
        .header("content-type", "application/json")
        .body(json_body(&super_admin_json()))
        .unwrap();
    app.request(req).await;

    let req = Request::post("/login")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(axum::body::Body::from(format!(
            "username=admin&password={VALID_PASSWORD}"
        )))
        .unwrap();
    let response = app.request(req).await;
    assert_eq!(response.status(), http::StatusCode::OK);
    let set_cookie = response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        set_cookie.contains("sid="),
        "Expected session cookie to be set"
    );
}

#[tokio::test]
async fn test_get_meta_issuer() {
    let app = common::TestApp::new().await;
    let req = Request::get("/meta/issuer")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::OK);
    let value: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(value.as_str().unwrap().starts_with("http"));
}

#[tokio::test]
async fn test_get_meta_jwk() {
    let app = common::TestApp::new().await;
    let req = Request::get("/meta/jwk")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::OK);
    let jwk: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(jwk.get("kty").is_some(), "JWK should contain key type");
}
