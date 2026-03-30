mod common;

use http::Request;

fn json_body(json: &str) -> axum::body::Body {
    axum::body::Body::from(json.to_string())
}

const VALID_PASSWORD: &str = "TestPassword123";

fn super_admin_json() -> String {
    format!(r#"{{"name": "admin", "full_name": "Super Admin", "password": "{VALID_PASSWORD}"}}"#)
}

/// Helper: create super admin and return a minted admin token.
async fn setup_with_admin(app: &common::TestApp) -> String {
    let req = Request::post("/users/super-admin")
        .header("content-type", "application/json")
        .body(json_body(&super_admin_json()))
        .unwrap();
    app.request(req).await;
    app.mint_token(0)
}

/// Helper: create a regular user and return its uid.
async fn create_user(app: &common::TestApp, token: &str, name: &str) -> u16 {
    let req = Request::post("/users")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(&format!(
            r#"{{"name": "{name}", "password": "{VALID_PASSWORD}", "groups": []}}"#
        )))
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CREATED);
    serde_json::from_str(&body).unwrap()
}

/// Helper: get user by uid.
async fn get_user(app: &common::TestApp, token: &str, uid: u16) -> serde_json::Value {
    let req = Request::get(format!("/users/{uid}"))
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::OK);
    serde_json::from_str(&body).unwrap()
}

// ── PATCH /users/{uid} ──────────────────────────────────────────────

#[tokio::test]
async fn test_patch_user_requires_auth() {
    let app = common::TestApp::new().await;

    let req = Request::patch("/users/1")
        .header("content-type", "application/json")
        .body(json_body(r#"{"name": "newname"}"#))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_patch_user_not_found() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;

    let req = Request::patch("/users/999")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(r#"{"name": "newname"}"#))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_patch_user_update_name() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;
    let uid = create_user(&app, &token, "testuser").await;

    let req = Request::patch(format!("/users/{uid}"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(r#"{"name": "renamed"}"#))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);

    let user = get_user(&app, &token, uid).await;
    assert_eq!(user["name"], "renamed");
}

#[tokio::test]
async fn test_patch_user_update_full_name() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;
    let uid = create_user(&app, &token, "testuser").await;

    let req = Request::patch(format!("/users/{uid}"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(r#"{"full_name": "Test User Full Name"}"#))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_patch_user_update_password() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;
    let uid = create_user(&app, &token, "testuser").await;

    let req = Request::patch(format!("/users/{uid}"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(r#"{"password": "NewPassword456"}"#))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_patch_user_duplicate_name() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;
    let uid = create_user(&app, &token, "testuser").await;
    create_user(&app, &token, "other").await;

    // Try to rename testuser to "other" which already exists
    let req = Request::patch(format!("/users/{uid}"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(r#"{"name": "other"}"#))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_patch_user_empty_body() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;
    let uid = create_user(&app, &token, "testuser").await;

    // Empty update (no fields) should succeed as a no-op
    let req = Request::patch(format!("/users/{uid}"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(r#"{}"#))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);

    let user = get_user(&app, &token, uid).await;
    assert_eq!(user["name"], "testuser");
}

#[tokio::test]
async fn test_patch_user_rename_to_same_name() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;
    let uid = create_user(&app, &token, "testuser").await;

    // Renaming to the same name should succeed
    let req = Request::patch(format!("/users/{uid}"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(r#"{"name": "testuser"}"#))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);
}

// ── PATCH /users/self ───────────────────────────────────────────────

#[tokio::test]
async fn test_patch_self_requires_auth() {
    let app = common::TestApp::new().await;

    let req = Request::patch("/users/self")
        .header("content-type", "application/json")
        .body(json_body(r#"{"name": "newname"}"#))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_patch_self_update_name() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;

    let req = Request::patch("/users/self")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(r#"{"name": "newadmin"}"#))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);

    let user = get_user(&app, &token, 0).await;
    assert_eq!(user["name"], "newadmin");
}

#[tokio::test]
async fn test_patch_self_update_password() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;

    let req = Request::patch("/users/self")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(r#"{"password": "NewPassword456"}"#))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_patch_self_duplicate_name() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;
    create_user(&app, &token, "other").await;

    // Try to rename self to "other" which already exists
    let req = Request::patch("/users/self")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(r#"{"name": "other"}"#))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_patch_self_as_regular_user() {
    let app = common::TestApp::new().await;
    let admin_token = setup_with_admin(&app).await;
    let uid = create_user(&app, &admin_token, "testuser").await;

    let user_token = app.mint_token(uid);

    let req = Request::patch("/users/self")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {user_token}"))
        .body(json_body(r#"{"full_name": "My Full Name"}"#))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);
}
