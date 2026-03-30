mod common;

use http::Request;
use std::path::Path;

const VALID_PASSWORD: &str = "TestPassword123";

fn super_admin_json() -> String {
    format!(r#"{{"name": "admin", "full_name": "Super Admin", "password": "{VALID_PASSWORD}"}}"#)
}

fn json_body(json: &str) -> axum::body::Body {
    axum::body::Body::from(json.to_string())
}

fn load_users_from_disk(path: &Path) -> Vec<serde_json::Value> {
    let content = std::fs::read_to_string(path).unwrap();
    let db: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(db["version"], "1");
    db["users"].as_array().unwrap().clone()
}

#[tokio::test]
async fn test_delete_self_requires_auth() {
    let app = common::TestApp::new().await;
    let req = Request::delete("/users/self")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_delete_self_success() {
    let app = common::TestApp::new().await;

    // Create super admin
    let req = Request::post("/users/super-admin")
        .header("content-type", "application/json")
        .body(json_body(&super_admin_json()))
        .unwrap();
    app.request(req).await;

    let admin_token = app.mint_token(0);

    // Create a regular user
    let req = Request::post("/users")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {admin_token}"))
        .body(json_body(&format!(
            r#"{{"name": "selfdelete", "password": "{VALID_PASSWORD}", "groups": []}}"#
        )))
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CREATED);
    let uid: u16 = serde_json::from_str(&body).unwrap();

    // Mint token for the new user and delete self
    let user_token = app.mint_token(uid);
    let req = Request::delete("/users/self")
        .header("authorization", format!("Bearer {user_token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);

    // Verify only super admin remains via API
    let req = Request::get("/users")
        .header("authorization", format!("Bearer {admin_token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (_, body) = app.request_body(req).await;
    let users: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
    assert_eq!(users.len(), 1);

    // Verify on-disk persistence
    let (users_path, _tempdir) = app.shutdown();
    let users = load_users_from_disk(&users_path);
    assert_eq!(users.len(), 1);
    assert_eq!(users[0]["name"], "admin");
}

#[tokio::test]
async fn test_delete_self_super_admin_forbidden() {
    let app = common::TestApp::new().await;

    // Create super admin
    let req = Request::post("/users/super-admin")
        .header("content-type", "application/json")
        .body(json_body(&super_admin_json()))
        .unwrap();
    app.request(req).await;

    let token = app.mint_token(0);

    // Super admin tries to delete self
    let req = Request::delete("/users/self")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::FORBIDDEN);
}
