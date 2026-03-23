mod common;

use http::Request;

fn json_body(json: &str) -> axum::body::Body {
    axum::body::Body::from(json.to_string())
}

fn client_count(app: &common::TestApp) -> usize {
    app.state.db.lock().unwrap().clients.query_all().count()
}

fn client_exists(app: &common::TestApp, name: &str) -> bool {
    app.state
        .db
        .lock()
        .unwrap()
        .clients
        .query_by_name(name)
        .is_some()
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

#[tokio::test]
async fn test_create_client_requires_auth() {
    let app = common::TestApp::new().await;
    let req = Request::put("/clients")
        .header("content-type", "application/json")
        .body(json_body(
            r#"{"name": "svc", "auth_method": {"type": "Secret"}, "groups": []}"#,
        ))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::FORBIDDEN);
    assert!(!client_exists(&app, "svc"));
}

#[tokio::test]
async fn test_create_client_with_secret() {
    let app = common::TestApp::new().await;
    let token = setup_admin(&app).await;

    let req = Request::put("/clients")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(
            r#"{"name": "my-service", "auth_method": {"type": "Secret"}, "groups": ["tech.flecs.admin"]}"#,
        ))
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CREATED, "body: {body}");

    let response: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(response["id"].is_string());
    assert_eq!(response["name"], "my-service");
    assert_eq!(response["auth_method"], "secret");
    assert!(response["secret"].is_string());
    assert!(!response["secret"].as_str().unwrap().is_empty());
    assert!(response["created_at"].is_string());
    assert!(response.get("certificate").is_none());
    assert!(response.get("private_key").is_none());

    // Verify client exists in database
    assert!(client_exists(&app, "my-service"));
    assert_eq!(client_count(&app), 1);
}

#[tokio::test]
async fn test_list_clients() {
    let app = common::TestApp::new().await;
    let token = setup_admin(&app).await;

    // Create two clients
    for name in ["svc-a", "svc-b"] {
        let req = Request::put("/clients")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(json_body(&format!(
                r#"{{"name": "{name}", "auth_method": {{"type": "Secret"}}, "groups": []}}"#
            )))
            .unwrap();
        let (status, _) = app.request_body(req).await;
        assert_eq!(status, http::StatusCode::CREATED);
    }

    // List
    let req = Request::get("/clients")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::OK);

    let clients: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
    assert_eq!(clients.len(), 2);

    // Verify no secrets in list
    for client in &clients {
        assert!(client.get("secret").is_none());
        assert!(client["auth_method"].is_string());
    }

    // Verify database state matches
    assert_eq!(client_count(&app), 2);
}

#[tokio::test]
async fn test_get_single_client() {
    let app = common::TestApp::new().await;
    let token = setup_admin(&app).await;

    // Create a client
    let req = Request::put("/clients")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(
            r#"{"name": "my-svc", "auth_method": {"type": "Secret"}, "groups": []}"#,
        ))
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CREATED);
    let created: serde_json::Value = serde_json::from_str(&body).unwrap();
    let cid = created["id"].as_str().unwrap();

    // Get by ID
    let req = Request::get(format!("/clients/{cid}"))
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::OK);

    let client: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(client["id"], cid);
    assert_eq!(client["name"], "my-svc");
    assert!(client.get("secret").is_none());

    // Verify database state
    assert!(client_exists(&app, "my-svc"));
}

#[tokio::test]
async fn test_get_nonexistent_client() {
    let app = common::TestApp::new().await;
    let token = setup_admin(&app).await;

    let req = Request::get("/clients/00000000-0000-0000-0000-000000000000")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_client() {
    let app = common::TestApp::new().await;
    let token = setup_admin(&app).await;

    // Create
    let req = Request::put("/clients")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(
            r#"{"name": "to-delete", "auth_method": {"type": "Secret"}, "groups": []}"#,
        ))
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CREATED);
    let created: serde_json::Value = serde_json::from_str(&body).unwrap();
    let cid = created["id"].as_str().unwrap();

    // Delete
    let req = Request::delete(format!("/clients/{cid}"))
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NO_CONTENT);

    // Verify gone via API
    let req = Request::get(format!("/clients/{cid}"))
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NOT_FOUND);

    // Verify gone in database
    assert!(!client_exists(&app, "to-delete"));
    assert_eq!(client_count(&app), 0);
}

#[tokio::test]
async fn test_duplicate_client_name() {
    let app = common::TestApp::new().await;
    let token = setup_admin(&app).await;

    let client_json = r#"{"name": "dup-svc", "auth_method": {"type": "Secret"}, "groups": []}"#;

    // First creation
    let req = Request::put("/clients")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(client_json))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CREATED);

    // Duplicate
    let req = Request::put("/clients")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(client_json))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CONFLICT);

    // Verify still only one client in database
    assert_eq!(client_count(&app), 1);
}

#[tokio::test]
async fn test_delete_nonexistent_client() {
    let app = common::TestApp::new().await;
    let token = setup_admin(&app).await;

    let req = Request::delete("/clients/00000000-0000-0000-0000-000000000000")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_read_only_client() {
    let ro_client_id = "11111111-1111-1111-1111-111111111111";

    let app = common::TestApp::new_with_setup(|dir| {
        let password_hash =
            user_manager::model::password::Password::new("DummyPassword123!").unwrap();
        let password_json = serde_json::to_value(&password_hash).unwrap();
        let ro_json = serde_json::json!({
            "version": "1",
            "clients": [{
                "id": ro_client_id,
                "name": "ro-client",
                "auth_method": {"type": "Secret", "secret": password_json},
                "groups": [],
                "created_at": "2026-01-01T00:00:00Z"
            }]
        });
        std::fs::write(
            dir.join("ro_clients.json"),
            serde_json::to_string(&ro_json).unwrap(),
        )
        .unwrap();
    })
    .await;
    let token = setup_admin(&app).await;

    // Read-only client should appear in list
    let req = Request::get("/clients")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::OK);
    let clients: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
    assert_eq!(clients.len(), 1);
    assert_eq!(clients[0]["name"], "ro-client");

    // Deleting read-only client should return 403
    let req = Request::delete(format!("/clients/{ro_client_id}"))
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::FORBIDDEN);

    // Verify read-only client is still in database
    assert!(client_exists(&app, "ro-client"));
    assert_eq!(client_count(&app), 1);
}

#[tokio::test]
async fn test_create_client_name_conflicts_with_read_only() {
    let app = common::TestApp::new_with_setup(|dir| {
        let password_hash =
            user_manager::model::password::Password::new("DummyPassword123!").unwrap();
        let password_json = serde_json::to_value(&password_hash).unwrap();
        let ro_json = serde_json::json!({
            "version": "1",
            "clients": [{
                "id": "22222222-2222-2222-2222-222222222222",
                "name": "reserved-name",
                "auth_method": {"type": "Secret", "secret": password_json},
                "groups": [],
                "created_at": "2026-01-01T00:00:00Z"
            }]
        });
        std::fs::write(
            dir.join("ro_clients.json"),
            serde_json::to_string(&ro_json).unwrap(),
        )
        .unwrap();
    })
    .await;
    let token = setup_admin(&app).await;

    // Creating client with same name as read-only should return 409
    let req = Request::put("/clients")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(
            r#"{"name": "reserved-name", "auth_method": {"type": "Secret"}, "groups": []}"#,
        ))
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CONFLICT);

    // Verify no new writable client was created (only the read-only one exists)
    assert_eq!(client_count(&app), 1);
}

#[tokio::test]
async fn test_list_clients_requires_auth() {
    let app = common::TestApp::new().await;
    let req = Request::get("/clients")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_create_client_forbidden_when_assigning_groups_caller_does_not_have() {
    let app = common::TestApp::new().await;
    let token = setup_admin(&app).await;

    // Admin has tech.flecs.admin (+ inherited developer, technician, operator),
    // but does NOT have "custom.group".
    let req = Request::put("/clients")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(
            r#"{"name": "bad-client", "auth_method": {"type": "Secret"}, "groups": ["custom.group"]}"#,
        ))
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::FORBIDDEN, "body: {body}");
    assert!(body.contains("custom.group"));

    // Verify no client was created in database
    assert!(!client_exists(&app, "bad-client"));
    assert_eq!(client_count(&app), 0);
}

#[tokio::test]
async fn test_create_client_with_caller_groups_succeeds() {
    let app = common::TestApp::new().await;
    let token = setup_admin(&app).await;

    // Admin has tech.flecs.admin and inherited tech.flecs.developer — both should be allowed
    let req = Request::put("/clients")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(json_body(
            r#"{"name": "good-client", "auth_method": {"type": "Secret"}, "groups": ["tech.flecs.admin", "tech.flecs.developer"]}"#,
        ))
        .unwrap();
    let (status, body) = app.request_body(req).await;
    assert_eq!(status, http::StatusCode::CREATED, "body: {body}");

    // Verify client exists in database with correct groups
    let db = app.state.db.lock().unwrap();
    let client = db.clients.query_by_name("good-client").unwrap();
    let group_names: std::collections::HashSet<&str> =
        client.groups.iter().map(|g| g.as_ref()).collect();
    assert!(group_names.contains("tech.flecs.admin"));
    assert!(group_names.contains("tech.flecs.developer"));
    assert_eq!(group_names.len(), 2);
}
