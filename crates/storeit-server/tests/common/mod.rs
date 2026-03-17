use std::sync::Arc;

use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use storeit_auth::{MockAuthProvider, OidcUserInfo};
use storeit_domain::entities::{CreateLocalUser, CreateUser, GroupRole, Session};
use storeit_domain::repositories::{
    GroupRepository, SessionRepository, SettingsRepository, UserGroupRepository, UserRepository,
};
use storeit_server::app_state::AppState;
use storeit_server::extractors::SESSION_COOKIE;
use storeit_server::router::build_router;

/// Known test session ID — pre-seeded in every TestApp.
const TEST_SESSION_ID: &str = "test-session-id-for-integration-tests-00000000000000000000000000";
/// A session that is seeded but already expired.
pub const EXPIRED_SESSION_ID: &str = "expired-session-id-for-integration-tests-0000000000000000000";
const TEST_GROUP_PREFIX: &str = "storeit:";

pub struct TestApp {
    pub addr: String,
    pub client: reqwest::Client,
    _tempdir: tempfile::TempDir,
}

impl TestApp {
    pub async fn spawn() -> Self {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect to in-memory sqlite");

        let db = storeit_db_sqlite::SqliteDb::new(pool.clone());
        db.migrate().await.expect("run migrations");

        let tempdir = tempfile::TempDir::new().expect("create temp dir");

        // Build repos
        let user_repo = Arc::new(storeit_db_sqlite::user_repo::SqliteUserRepository::new(
            pool.clone(),
        ));
        let group_repo = Arc::new(storeit_db_sqlite::group_repo::SqliteGroupRepository::new(
            pool.clone(),
        ));
        let user_group_repo = Arc::new(
            storeit_db_sqlite::user_group_repo::SqliteUserGroupRepository::new(pool.clone()),
        );
        let session_repo =
            Arc::new(storeit_db_sqlite::session_repo::SqliteSessionRepository::new(pool.clone()));
        let settings_repo: Arc<dyn SettingsRepository> =
            Arc::new(storeit_db_sqlite::settings_repo::SqliteSettingsRepository::new(pool.clone()));

        // Auto-provision test user, group, membership, and session
        let user = user_repo
            .upsert_by_external_id(CreateUser {
                external_id: "test-user-001".into(),
                email: "test@example.com".into(),
                display_name: "Test User".into(),
            })
            .await
            .expect("create test user");

        let group = group_repo
            .get_or_create_by_name("default")
            .await
            .expect("get default group");

        user_group_repo
            .set_memberships(user.id, vec![(group.id, GroupRole::Owner)])
            .await
            .expect("set test membership");

        session_repo
            .create(Session {
                id: TEST_SESSION_ID.into(),
                user_id: user.id,
                active_group_id: group.id,
                expires_at: Utc::now() + Duration::hours(24),
                created_at: Utc::now(),
            })
            .await
            .expect("create test session");

        // Also seed an expired session for testing the expiry check path
        session_repo
            .create(Session {
                id: EXPIRED_SESSION_ID.into(),
                user_id: user.id,
                active_group_id: group.id,
                expires_at: Utc::now() - Duration::hours(1), // already expired
                created_at: Utc::now() - Duration::hours(25),
            })
            .await
            .expect("create expired test session");

        // Mock auth provider
        let auth_provider = Arc::new(MockAuthProvider::new(
            OidcUserInfo {
                external_id: "test-user-001".into(),
                email: "test@example.com".into(),
                display_name: "Test User".into(),
                groups: vec![format!("{TEST_GROUP_PREFIX}default")],
            },
            TEST_GROUP_PREFIX,
        ));

        let item_identifier: Option<Arc<dyn storeit_domain::storage::ItemIdentifier>> =
            Some(Arc::new(storeit_ai::MockItemIdentifier));

        let image_path = tempdir.path().to_string_lossy().into_owned();

        let state = AppState {
            db_pool: pool.clone(),
            location_repo: Arc::new(
                storeit_db_sqlite::location_repo::SqliteLocationRepository::new(pool.clone()),
            ),
            container_repo: Arc::new(
                storeit_db_sqlite::container_repo::SqliteContainerRepository::new(pool.clone()),
            ),
            item_repo: Arc::new(storeit_db_sqlite::item_repo::SqliteItemRepository::new(
                pool.clone(),
            )),
            photo_repo: Arc::new(storeit_db_sqlite::photo_repo::SqlitePhotoRepository::new(
                pool.clone(),
            )),
            nfc_tag_repo: Arc::new(
                storeit_db_sqlite::nfc_tag_repo::SqliteNfcTagRepository::new(pool.clone()),
            ),
            search_repo: Arc::new(storeit_db_sqlite::search_repo::SqliteSearchRepository::new(
                pool,
            )),
            settings_repo,
            image_storage: std::sync::RwLock::new(Arc::new(
                storeit_storage_fs::FsImageStorage::new(&image_path),
            )),
            image_storage_path: std::sync::RwLock::new(image_path),
            env_image_path: false,

            // AI
            item_identifier,

            // Auth
            user_repo,
            group_repo,
            user_group_repo,
            session_repo,
            auth_provider: Some(auth_provider),
            auth_mode: storeit_auth::AuthMode::Oidc,
            session_secret: "test-secret-must-be-at-least-32-ch".into(),
            session_ttl_hours: 24,
            backup_jobs: dashmap::DashMap::new(),
            restore_jobs: dashmap::DashMap::new(),
        };

        let app = build_router(Arc::new(state));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind to random port");
        let addr = format!("http://{}", listener.local_addr().unwrap());

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Build a client with the test session cookie on all requests
        let client = reqwest::Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::COOKIE,
                    format!("{SESSION_COOKIE}={TEST_SESSION_ID}")
                        .parse()
                        .unwrap(),
                );
                headers
            })
            .build()
            .expect("build reqwest client with session cookie");

        Self {
            addr,
            client,
            _tempdir: tempdir,
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.addr, path)
    }
}

/// A TestApp in local auth mode with an admin user — for testing admin endpoints.
pub struct AdminTestApp {
    pub addr: String,
    pub client: reqwest::Client,
    pub non_admin_client: reqwest::Client,
    pub settings_repo: Arc<dyn SettingsRepository>,
    _tempdir: tempfile::TempDir,
}

const ADMIN_SESSION_ID: &str = "admin-session-id-for-integration-tests-000000000000000000000000";
const NON_ADMIN_SESSION_ID: &str =
    "nonadm-session-id-for-integration-tests-000000000000000000000000";

impl AdminTestApp {
    pub async fn spawn() -> Self {
        Self::spawn_with_env_image_path(false).await
    }

    pub async fn spawn_with_env_image_path(env_image_path: bool) -> Self {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect to in-memory sqlite");

        let db = storeit_db_sqlite::SqliteDb::new(pool.clone());
        db.migrate().await.expect("run migrations");

        let tempdir = tempfile::TempDir::new().expect("create temp dir");

        let user_repo: Arc<dyn UserRepository> = Arc::new(
            storeit_db_sqlite::user_repo::SqliteUserRepository::new(pool.clone()),
        );
        let group_repo: Arc<dyn GroupRepository> = Arc::new(
            storeit_db_sqlite::group_repo::SqliteGroupRepository::new(pool.clone()),
        );
        let user_group_repo: Arc<dyn UserGroupRepository> = Arc::new(
            storeit_db_sqlite::user_group_repo::SqliteUserGroupRepository::new(pool.clone()),
        );
        let session_repo: Arc<dyn SessionRepository> =
            Arc::new(storeit_db_sqlite::session_repo::SqliteSessionRepository::new(pool.clone()));
        let settings_repo: Arc<dyn SettingsRepository> =
            Arc::new(storeit_db_sqlite::settings_repo::SqliteSettingsRepository::new(pool.clone()));

        let group = group_repo
            .get_or_create_by_name("default")
            .await
            .expect("get default group");

        // Create admin user
        let password_hash = storeit_auth::hash_password("admin123").unwrap();
        let admin_user = user_repo
            .create_local(CreateLocalUser {
                username: "admin".into(),
                email: "admin@test.com".into(),
                display_name: "Test Admin".into(),
                password_hash,
                is_admin: true,
            })
            .await
            .expect("create admin user");

        user_group_repo
            .add_member(admin_user.id, group.id, GroupRole::Owner)
            .await
            .expect("add admin to group");

        session_repo
            .create(Session {
                id: ADMIN_SESSION_ID.into(),
                user_id: admin_user.id,
                active_group_id: group.id,
                expires_at: Utc::now() + Duration::hours(24),
                created_at: Utc::now(),
            })
            .await
            .expect("create admin session");

        // Create non-admin user
        let non_admin_hash = storeit_auth::hash_password("user123").unwrap();
        let non_admin_user = user_repo
            .create_local(CreateLocalUser {
                username: "user".into(),
                email: "user@test.com".into(),
                display_name: "Test User".into(),
                password_hash: non_admin_hash,
                is_admin: false,
            })
            .await
            .expect("create non-admin user");

        user_group_repo
            .add_member(non_admin_user.id, group.id, GroupRole::Member)
            .await
            .expect("add non-admin to group");

        session_repo
            .create(Session {
                id: NON_ADMIN_SESSION_ID.into(),
                user_id: non_admin_user.id,
                active_group_id: group.id,
                expires_at: Utc::now() + Duration::hours(24),
                created_at: Utc::now(),
            })
            .await
            .expect("create non-admin session");

        let image_path = tempdir.path().to_string_lossy().into_owned();

        let state = AppState {
            db_pool: pool.clone(),
            location_repo: Arc::new(
                storeit_db_sqlite::location_repo::SqliteLocationRepository::new(pool.clone()),
            ),
            container_repo: Arc::new(
                storeit_db_sqlite::container_repo::SqliteContainerRepository::new(pool.clone()),
            ),
            item_repo: Arc::new(storeit_db_sqlite::item_repo::SqliteItemRepository::new(
                pool.clone(),
            )),
            photo_repo: Arc::new(storeit_db_sqlite::photo_repo::SqlitePhotoRepository::new(
                pool.clone(),
            )),
            nfc_tag_repo: Arc::new(
                storeit_db_sqlite::nfc_tag_repo::SqliteNfcTagRepository::new(pool.clone()),
            ),
            search_repo: Arc::new(storeit_db_sqlite::search_repo::SqliteSearchRepository::new(
                pool,
            )),
            settings_repo: settings_repo.clone(),
            image_storage: std::sync::RwLock::new(Arc::new(
                storeit_storage_fs::FsImageStorage::new(&image_path),
            )),
            image_storage_path: std::sync::RwLock::new(image_path),
            env_image_path,
            item_identifier: None,
            user_repo,
            group_repo,
            user_group_repo,
            session_repo,
            auth_provider: None,
            auth_mode: storeit_auth::AuthMode::Local,
            session_secret: "test-secret-must-be-at-least-32-ch".into(),
            session_ttl_hours: 24,
            backup_jobs: dashmap::DashMap::new(),
            restore_jobs: dashmap::DashMap::new(),
        };

        let app = build_router(Arc::new(state));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind to random port");
        let addr = format!("http://{}", listener.local_addr().unwrap());

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let client = reqwest::Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::COOKIE,
                    format!("{SESSION_COOKIE}={ADMIN_SESSION_ID}")
                        .parse()
                        .unwrap(),
                );
                headers
            })
            .build()
            .expect("build admin client");

        let non_admin_client = reqwest::Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::COOKIE,
                    format!("{SESSION_COOKIE}={NON_ADMIN_SESSION_ID}")
                        .parse()
                        .unwrap(),
                );
                headers
            })
            .build()
            .expect("build non-admin client");

        Self {
            addr,
            client,
            non_admin_client,
            settings_repo,
            _tempdir: tempdir,
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.addr, path)
    }
}
