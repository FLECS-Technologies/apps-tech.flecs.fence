use axum::Router;
use http::Request;
use http_body_util::BodyExt;
use tempfile::TempDir;
use tower::ServiceExt;
use user_manager::config::Config;
use user_manager::model::user::UserId;
use user_manager::state::{self, AppState};

// Each test file compiles common.rs as its own submodule, so fields/methods
// only used by some test files appear unused in others.
#[allow(dead_code)]
pub struct TestApp {
    pub state: AppState,
    pub router: Router,
    pub users_path: std::path::PathBuf,
    tempdir: TempDir,
}

#[allow(dead_code)]
impl TestApp {
    pub async fn new() -> Self {
        Self::new_with_setup(|_| {}).await
    }

    /// Create a TestApp with a setup closure that runs before the app starts.
    /// The closure receives the tempdir path so it can pre-populate files.
    pub async fn new_with_setup(setup: impl FnOnce(&std::path::Path)) -> Self {
        let tempdir = TempDir::new().unwrap();

        let casbin_source = format!(
            "{}/../../docker/fs/var/local/lib/fence",
            env!("CARGO_MANIFEST_DIR")
        );
        let model_path = tempdir.path().join("casbin_model.conf");
        let policy_path = tempdir.path().join("casbin_policy.csv");
        std::fs::copy(format!("{casbin_source}/casbin_model.conf"), &model_path).unwrap();
        std::fs::copy(format!("{casbin_source}/casbin_policy.csv"), &policy_path).unwrap();

        setup(tempdir.path());

        let config = Config {
            database: user_manager::config::Database {
                users_path: tempdir.path().join("users.json"),
                groups_path: tempdir.path().join("groups.json"),
                clients_path: tempdir.path().join("clients.json"),
                ro_clients_path: tempdir.path().join("ro_clients.json"),
            },
            auth: user_manager::config::Auth {
                issuer_url: url::Url::parse("http://localhost").unwrap(),
                casbin_model_path: model_path,
                casbin_policy_path: policy_path,
            },
        };

        let enforcer = state::construct_enforcer(
            config.auth.casbin_model_path.clone(),
            config.auth.casbin_policy_path.clone(),
        )
        .await
        .unwrap();
        let app_state = AppState::new(enforcer, &config);
        let router = user_manager::router::build_router(app_state.clone());

        let users_path = config.database.users_path.clone();
        Self {
            state: app_state,
            router,
            users_path,
            tempdir,
        }
    }

    /// Drop the application state (triggering DB persistence) and return the
    /// tempdir handle so the on-disk files can be inspected before cleanup.
    pub fn shutdown(self) -> (std::path::PathBuf, TempDir) {
        let users_path = self.users_path.clone();
        let tempdir = self.tempdir;
        drop(self.state);
        drop(self.router);
        (users_path, tempdir)
    }

    pub async fn request(
        &self,
        request: Request<axum::body::Body>,
    ) -> http::Response<axum::body::Body> {
        self.router.clone().oneshot(request).await.unwrap()
    }

    pub async fn request_body(
        &self,
        request: Request<axum::body::Body>,
    ) -> (http::StatusCode, String) {
        let response = self.request(request).await;
        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body = String::from_utf8_lossy(&body).to_string();
        (status, body)
    }

    /// Mint a JWT token for the given user. Uses `token::issue` which derives
    /// roles from the user's groups in the database.
    pub fn mint_token(&self, user_id: UserId) -> String {
        use oxide_auth::primitives::grant::Grant;
        let issuer = self.state.issuer.lock().unwrap();
        let grant = Grant {
            owner_id: user_id.to_string(),
            client_id: "flecs".to_string(),
            redirect_uri: url::Url::parse("https://localhost/").unwrap(),
            scope: "admin".parse().unwrap(),
            until: chrono::Utc::now() + chrono::Duration::hours(1),
            extensions: Default::default(),
        };
        let token = user_manager::token::issue(
            grant,
            issuer.url.clone(),
            issuer.jwk.common.key_id.clone(),
            &issuer.encoding_key,
            self.state.db.clone(),
        )
        .unwrap();
        token.token
    }
}
