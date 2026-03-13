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
    let req = Request::put("/users")
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

/// Helper: get roles for a user via GET /users/{uid}/roles.
async fn get_roles(app: &common::TestApp, token: &str, uid: u16) -> Vec<String> {
    let req = Request::get(format!("/users/{uid}/roles"))
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::OK);
    serde_json::from_str(&body).unwrap()
}

// ── PUT /users/{uid}/roles (bulk assign) ────────────────────────────

#[tokio::test]
async fn test_put_roles_requires_auth() {
    let app = common::TestApp::new().await;

    let req = Request::put("/users/1/roles")
        .header("content-type", "application/json")
        .body(json_body(r#"["tech.flecs.admin"]"#))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_put_roles_user_not_found() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;

    let req = Request::put("/users/999/roles")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(r#"["tech.flecs.admin"]"#))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_put_roles_success() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;
    let uid = create_user(&app, &token, "testuser").await;

    // Assign roles
    let req = Request::put(format!("/users/{uid}/roles"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(
            r#"["tech.flecs.developer", "tech.flecs.operator"]"#,
        ))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);

    // Verify roles were updated
    let req = Request::get(format!("/users/{uid}"))
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::OK);

    let user: serde_json::Value = serde_json::from_str(&body).unwrap();
    let groups: Vec<&str> = user["groups"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g.as_str().unwrap())
        .collect();
    assert!(groups.contains(&"tech.flecs.developer"));
    assert!(groups.contains(&"tech.flecs.operator"));
    assert_eq!(groups.len(), 2);
}

#[tokio::test]
async fn test_put_roles_replaces_existing() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;
    let uid = create_user(&app, &token, "testuser").await;

    // Assign initial roles
    let req = Request::put(format!("/users/{uid}/roles"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(
            r#"["tech.flecs.developer", "tech.flecs.operator"]"#,
        ))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);

    // Replace with different roles
    let req = Request::put(format!("/users/{uid}/roles"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(r#"["tech.flecs.admin"]"#))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);

    // Verify only the new role is present
    let req = Request::get(format!("/users/{uid}"))
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (_, body) = app.request_body(req).await;

    let user: serde_json::Value = serde_json::from_str(&body).unwrap();
    let groups: Vec<&str> = user["groups"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g.as_str().unwrap())
        .collect();
    assert_eq!(groups, vec!["tech.flecs.admin"]);
}

#[tokio::test]
async fn test_put_roles_empty_clears_roles() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;
    let uid = create_user(&app, &token, "testuser").await;

    // Assign roles
    let req = Request::put(format!("/users/{uid}/roles"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(r#"["tech.flecs.developer"]"#))
        .unwrap();
    app.request(req).await;

    // Clear roles
    let req = Request::put(format!("/users/{uid}/roles"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(r#"[]"#))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);

    // Verify empty
    let req = Request::get(format!("/users/{uid}"))
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (_, body) = app.request_body(req).await;

    let user: serde_json::Value = serde_json::from_str(&body).unwrap();
    let groups = user["groups"].as_array().unwrap();
    assert!(groups.is_empty());
}

// ── GET /users/{uid}/roles ──────────────────────────────────────────

#[tokio::test]
async fn test_get_roles_requires_auth() {
    let app = common::TestApp::new().await;

    let req = Request::get("/users/1/roles")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_roles_user_not_found() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;

    let req = Request::get("/users/999/roles")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_roles_success() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;
    let uid = create_user(&app, &token, "testuser").await;

    // Assign roles first
    let req = Request::put(format!("/users/{uid}/roles"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(
            r#"["tech.flecs.developer", "tech.flecs.operator"]"#,
        ))
        .unwrap();
    app.request(req).await;

    // Get roles
    let roles = get_roles(&app, &token, uid).await;
    assert_eq!(roles.len(), 2);
    assert!(roles.contains(&"tech.flecs.developer".to_string()));
    assert!(roles.contains(&"tech.flecs.operator".to_string()));
}

#[tokio::test]
async fn test_get_roles_empty() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;
    let uid = create_user(&app, &token, "testuser").await;

    let roles = get_roles(&app, &token, uid).await;
    assert!(roles.is_empty());
}

// ── PUT /users/{uid}/roles/{role} (single assign) ───────────────────

#[tokio::test]
async fn test_put_single_role_requires_auth() {
    let app = common::TestApp::new().await;

    let req = Request::put("/users/1/roles/tech.flecs.admin")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_put_single_role_user_not_found() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;

    let req = Request::put("/users/999/roles/tech.flecs.admin")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_put_single_role_success() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;
    let uid = create_user(&app, &token, "testuser").await;

    // Assign single role
    let req = Request::put(format!("/users/{uid}/roles/tech.flecs.developer"))
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);

    // Verify
    let roles = get_roles(&app, &token, uid).await;
    assert_eq!(roles, vec!["tech.flecs.developer"]);
}

#[tokio::test]
async fn test_put_single_role_conflict() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;
    let uid = create_user(&app, &token, "testuser").await;

    // Assign role
    let req = Request::put(format!("/users/{uid}/roles/tech.flecs.developer"))
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    app.request(req).await;

    // Assign same role again
    let req = Request::put(format!("/users/{uid}/roles/tech.flecs.developer"))
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_put_single_role_adds_to_existing() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;
    let uid = create_user(&app, &token, "testuser").await;

    // Assign first role
    let req = Request::put(format!("/users/{uid}/roles/tech.flecs.developer"))
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    app.request(req).await;

    // Assign second role
    let req = Request::put(format!("/users/{uid}/roles/tech.flecs.operator"))
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);

    // Verify both roles present
    let roles = get_roles(&app, &token, uid).await;
    assert_eq!(roles.len(), 2);
    assert!(roles.contains(&"tech.flecs.developer".to_string()));
    assert!(roles.contains(&"tech.flecs.operator".to_string()));
}

// ── DELETE /users/{uid}/roles/{role} ────────────────────────────────

#[tokio::test]
async fn test_delete_role_requires_auth() {
    let app = common::TestApp::new().await;

    let req = Request::delete("/users/1/roles/tech.flecs.admin")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_delete_role_user_not_found() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;

    let req = Request::delete("/users/999/roles/tech.flecs.admin")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_role_not_assigned() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;
    let uid = create_user(&app, &token, "testuser").await;

    let req = Request::delete(format!("/users/{uid}/roles/tech.flecs.admin"))
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_role_success() {
    let app = common::TestApp::new().await;
    let token = setup_with_admin(&app).await;
    let uid = create_user(&app, &token, "testuser").await;

    // Assign two roles
    let req = Request::put(format!("/users/{uid}/roles"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(
            r#"["tech.flecs.developer", "tech.flecs.operator"]"#,
        ))
        .unwrap();
    app.request(req).await;

    // Delete one role
    let req = Request::delete(format!("/users/{uid}/roles/tech.flecs.developer"))
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);

    // Verify only the other role remains
    let roles = get_roles(&app, &token, uid).await;
    assert_eq!(roles, vec!["tech.flecs.operator"]);
}
