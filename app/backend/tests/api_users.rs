mod common;

use http::Request;
use std::path::Path;

fn json_body(json: &str) -> axum::body::Body {
    axum::body::Body::from(json.to_string())
}

fn load_users_from_disk(path: &Path) -> Vec<serde_json::Value> {
    let content = std::fs::read_to_string(path).unwrap();
    let db: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(db["version"], "1");
    db["users"].as_array().unwrap().clone()
}

const VALID_PASSWORD: &str = "TestPassword123";

fn super_admin_json() -> String {
    format!(
        r#"{{"name": "admin", "full_name": "Super Admin", "password": "{VALID_PASSWORD}"}}"#
    )
}

#[tokio::test]
async fn test_get_super_admin_not_found() {
    let app = common::TestApp::new().await;
    let req = Request::get("/users/super-admin")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_super_admin() {
    let app = common::TestApp::new().await;
    let req = Request::post("/users/super-admin")
        .header("content-type", "application/json")
        .body(json_body(&super_admin_json()))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::OK);

    // Verify super admin now exists
    let req = Request::get("/users/super-admin")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);

    // Verify on-disk persistence
    let (users_path, _tempdir) = app.shutdown();
    let users = load_users_from_disk(&users_path);
    assert_eq!(users.len(), 1);
    assert_eq!(users[0]["name"], "admin");
    assert_eq!(users[0]["id"], 0);
}

#[tokio::test]
async fn test_create_super_admin_conflict() {
    let app = common::TestApp::new().await;

    let req = Request::post("/users/super-admin")
        .header("content-type", "application/json")
        .body(json_body(&super_admin_json()))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::OK);

    // Second creation should conflict
    let req = Request::post("/users/super-admin")
        .header("content-type", "application/json")
        .body(json_body(&super_admin_json()))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_users_requires_auth() {
    let app = common::TestApp::new().await;
    let req = Request::get("/users")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_users_with_auth() {
    let app = common::TestApp::new().await;

    // Create super admin first
    let req = Request::post("/users/super-admin")
        .header("content-type", "application/json")
        .body(json_body(&super_admin_json()))
        .unwrap();
    app.request(req).await;

    // Mint token for super admin (id=0)
    let token = app.mint_token(0);

    let req = Request::get("/users")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::OK);

    let users: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0]["name"], "admin");
}

#[tokio::test]
async fn test_put_user_requires_auth() {
    let app = common::TestApp::new().await;
    let req = Request::put("/users")
        .header("content-type", "application/json")
        .body(json_body(
            &format!(r#"{{"name": "testuser", "password": "{VALID_PASSWORD}", "groups": []}}"#),
        ))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_put_user_with_auth() {
    let app = common::TestApp::new().await;

    // Create super admin
    let req = Request::post("/users/super-admin")
        .header("content-type", "application/json")
        .body(json_body(&super_admin_json()))
        .unwrap();
    app.request(req).await;

    let token = app.mint_token(0);

    // Create a new user
    let req = Request::put("/users")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(
            &format!(r#"{{"name": "testuser", "password": "{VALID_PASSWORD}", "groups": []}}"#),
        ))
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CREATED, "body: {body}");

    // Verify the user appears in the list
    let req = Request::get("/users")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::OK);

    let users: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
    assert_eq!(users.len(), 2);

    // Verify on-disk persistence
    let (users_path, _tempdir) = app.shutdown();
    let users = load_users_from_disk(&users_path);
    assert_eq!(users.len(), 2);
    let names: Vec<&str> = users.iter().map(|u| u["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"admin"));
    assert!(names.contains(&"testuser"));
}

#[tokio::test]
async fn test_put_user_duplicate_name() {
    let app = common::TestApp::new().await;

    // Create super admin
    let req = Request::post("/users/super-admin")
        .header("content-type", "application/json")
        .body(json_body(&super_admin_json()))
        .unwrap();
    app.request(req).await;

    let token = app.mint_token(0);
    let user_json = format!(r#"{{"name": "testuser", "password": "{VALID_PASSWORD}", "groups": []}}"#);

    // Create user
    let req = Request::put("/users")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(&user_json))
        .unwrap();
    app.request(req).await;

    // Create duplicate
    let req = Request::put("/users")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(&user_json))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_delete_user() {
    let app = common::TestApp::new().await;

    // Create super admin
    let req = Request::post("/users/super-admin")
        .header("content-type", "application/json")
        .body(json_body(&super_admin_json()))
        .unwrap();
    app.request(req).await;

    let token = app.mint_token(0);

    // Create a user to delete
    let req = Request::put("/users")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(
            &format!(r#"{{"name": "to_delete", "password": "{VALID_PASSWORD}", "groups": []}}"#),
        ))
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CREATED);
    let uid: u16 = serde_json::from_str(&body).unwrap();

    // Delete the user
    let req = Request::delete(format!("/users/{uid}"))
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);

    // Verify user is gone
    let req = Request::get("/users")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (_, body) = app.request_body(req).await;
    let users: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
    assert_eq!(users.len(), 1); // Only super admin remains

    // Verify on-disk persistence
    let (users_path, _tempdir) = app.shutdown();
    let users = load_users_from_disk(&users_path);
    assert_eq!(users.len(), 1);
    assert_eq!(users[0]["name"], "admin");
}

#[tokio::test]
async fn test_delete_super_admin_forbidden() {
    let app = common::TestApp::new().await;

    // Create super admin
    let req = Request::post("/users/super-admin")
        .header("content-type", "application/json")
        .body(json_body(&super_admin_json()))
        .unwrap();
    app.request(req).await;

    let token = app.mint_token(0);

    // Try to delete super admin (id=0)
    let req = Request::delete("/users/0")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_delete_nonexistent_user() {
    let app = common::TestApp::new().await;

    // Create super admin
    let req = Request::post("/users/super-admin")
        .header("content-type", "application/json")
        .body(json_body(&super_admin_json()))
        .unwrap();
    app.request(req).await;

    let token = app.mint_token(0);

    // Try to delete non-existent user
    let req = Request::delete("/users/999")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NOT_FOUND);
}
