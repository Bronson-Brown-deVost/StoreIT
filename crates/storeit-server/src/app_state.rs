use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use dashmap::DashMap;
use sqlx::SqlitePool;
use storeit_auth::{AuthMode, AuthProvider};
use storeit_domain::entities::{CreateLocalUser, GroupRole};
use storeit_domain::repositories::*;
use storeit_domain::storage::{ImageStorage, ItemIdentifier};

use crate::interchange::ProgressReporter;

// -- Job types for backup / restore --

pub struct BackupJob {
    pub status: Mutex<String>,
    pub progress: AtomicU64,
    pub total: AtomicU64,
    pub error: Mutex<Option<String>>,
    pub archive_path: Mutex<Option<PathBuf>>,
}

impl Default for BackupJob {
    fn default() -> Self {
        Self::new()
    }
}

impl BackupJob {
    pub fn new() -> Self {
        Self {
            status: Mutex::new("pending".into()),
            progress: AtomicU64::new(0),
            total: AtomicU64::new(0),
            error: Mutex::new(None),
            archive_path: Mutex::new(None),
        }
    }

    pub fn set_status(&self, s: &str) {
        *self.status.lock().unwrap() = s.into();
    }

    pub fn status(&self) -> String {
        self.status.lock().unwrap().clone()
    }

    pub fn inc_progress(&self) {
        self.progress.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_error(&self, msg: String) {
        *self.error.lock().unwrap() = Some(msg);
        self.set_status("failed");
    }

    pub fn set_complete(&self, path: PathBuf) {
        *self.archive_path.lock().unwrap() = Some(path);
        self.set_status("complete");
    }
}

impl ProgressReporter for BackupJob {
    fn set_total(&self, total: u64) {
        self.total.store(total, Ordering::Relaxed);
    }

    fn inc_progress(&self) {
        self.progress.fetch_add(1, Ordering::Relaxed);
    }

    fn set_status(&self, status: &str) {
        *self.status.lock().unwrap() = status.into();
    }
}

pub struct RestoreJob {
    pub status: Mutex<String>,
    pub progress: AtomicU64,
    pub total: AtomicU64,
    pub error: Mutex<Option<String>>,
}

impl Default for RestoreJob {
    fn default() -> Self {
        Self::new()
    }
}

impl RestoreJob {
    pub fn new() -> Self {
        Self {
            status: Mutex::new("pending".into()),
            progress: AtomicU64::new(0),
            total: AtomicU64::new(0),
            error: Mutex::new(None),
        }
    }

    pub fn set_status(&self, s: &str) {
        *self.status.lock().unwrap() = s.into();
    }

    pub fn status(&self) -> String {
        self.status.lock().unwrap().clone()
    }

    pub fn inc_progress(&self) {
        self.progress.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_error(&self, msg: String) {
        *self.error.lock().unwrap() = Some(msg);
        self.set_status("failed");
    }

    pub fn set_complete(&self) {
        self.set_status("complete");
    }
}

impl ProgressReporter for RestoreJob {
    fn set_total(&self, total: u64) {
        self.total.store(total, Ordering::Relaxed);
    }

    fn inc_progress(&self) {
        self.progress.fetch_add(1, Ordering::Relaxed);
    }

    fn set_status(&self, status: &str) {
        *self.status.lock().unwrap() = status.into();
    }
}

pub struct AppState {
    pub db_pool: SqlitePool,
    pub location_repo: Arc<dyn LocationRepository>,
    pub container_repo: Arc<dyn ContainerRepository>,
    pub item_repo: Arc<dyn ItemRepository>,
    pub photo_repo: Arc<dyn PhotoRepository>,
    pub nfc_tag_repo: Arc<dyn NfcTagRepository>,
    pub search_repo: Arc<dyn SearchRepository>,
    pub settings_repo: Arc<dyn SettingsRepository>,

    // Image storage — swappable at runtime when admin changes the path
    pub image_storage: std::sync::RwLock<Arc<dyn ImageStorage>>,
    pub image_storage_path: std::sync::RwLock<String>,
    /// True when the image path was set via the STOREIT_IMAGE_PATH env var (read-only in UI).
    pub env_image_path: bool,

    // AI identification
    pub item_identifier: Option<Arc<dyn ItemIdentifier>>,

    // Auth
    pub user_repo: Arc<dyn UserRepository>,
    pub group_repo: Arc<dyn GroupRepository>,
    pub user_group_repo: Arc<dyn UserGroupRepository>,
    pub session_repo: Arc<dyn SessionRepository>,
    pub auth_provider: Option<Arc<dyn AuthProvider>>,
    pub auth_mode: AuthMode,
    pub session_secret: String,
    pub session_ttl_hours: u64,

    // Backup / restore jobs
    pub backup_jobs: DashMap<String, Arc<BackupJob>>,
    pub restore_jobs: DashMap<String, Arc<RestoreJob>>,
}

impl AppState {
    /// Get the underlying database pool.
    pub fn db_pool(&self) -> &SqlitePool {
        &self.db_pool
    }

    /// Get the current image storage backend.
    pub fn image_storage(&self) -> Arc<dyn ImageStorage> {
        self.image_storage.read().unwrap().clone()
    }

    /// Get the current image storage path.
    pub fn image_storage_path(&self) -> String {
        self.image_storage_path.read().unwrap().clone()
    }

    /// Swap the image storage backend and path at runtime.
    pub fn swap_image_storage(&self, path: String, storage: Arc<dyn ImageStorage>) {
        *self.image_storage_path.write().unwrap() = path;
        *self.image_storage.write().unwrap() = storage;
    }

    pub async fn new(config: &crate::config::Config) -> anyhow::Result<Self> {
        // Ensure data directories exist before connecting to the database
        let image_dir = std::path::Path::new(&config.image_storage_path);
        if let Some(parent) = image_dir.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::create_dir_all(image_dir)?;

        // Ensure the SQLite database parent directory exists
        if let Some(parent) = config
            .database_url
            .strip_prefix("sqlite:")
            .and_then(|s| s.split('?').next())
            .map(std::path::Path::new)
            .and_then(|p| p.parent())
            .filter(|p| !p.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }

        let pool = SqlitePool::connect(&config.database_url).await?;

        let db = storeit_db_sqlite::SqliteDb::new(pool.clone());
        db.migrate().await?;

        // Build the auth provider (real OIDC or local auth)
        let (auth_mode, auth_provider): (AuthMode, Option<Arc<dyn AuthProvider>>) =
            if let Some(issuer_url) = &config.auth_issuer_url {
                let auth_config = storeit_auth::AuthConfig {
                    issuer_url: issuer_url.clone(),
                    client_id: config.auth_client_id.clone(),
                    client_secret: config.auth_client_secret.clone(),
                    redirect_uri: config.auth_redirect_uri.clone(),
                    group_prefix: config.auth_group_prefix.clone(),
                };
                (
                    AuthMode::Oidc,
                    Some(Arc::new(
                        storeit_auth::oidc::OidcProvider::new(&auth_config).await?,
                    )),
                )
            } else {
                tracing::info!("STOREIT_AUTH_ISSUER not set — using local auth mode");
                (AuthMode::Local, None)
            };

        // Build AI identifier
        let item_identifier: Option<Arc<dyn ItemIdentifier>> = if let Some(api_key) =
            &config.anthropic_api_key
        {
            tracing::info!(
                "AI identification enabled via Anthropic API (model: {})",
                config.ai_model
            );
            Some(Arc::new(storeit_ai::AnthropicApiIdentifier::new(
                api_key.clone(),
                config.ai_model.clone(),
            )))
        } else {
            // Try to detect claude CLI on PATH
            match std::process::Command::new(&config.claude_code_path)
                .arg("--version")
                .output()
            {
                Ok(output) if output.status.success() => {
                    tracing::info!(
                        "AI identification enabled via claude CLI ({})",
                        config.claude_code_path
                    );
                    Some(Arc::new(storeit_ai::ClaudeCodeIdentifier::new(
                        config.claude_code_path.clone(),
                    )))
                }
                _ => {
                    tracing::warn!(
                        "AI identification disabled — no API key and claude CLI not found at '{}'",
                        config.claude_code_path
                    );
                    None
                }
            }
        };

        let user_repo: Arc<dyn UserRepository> = Arc::new(
            storeit_db_sqlite::user_repo::SqliteUserRepository::new(pool.clone()),
        );
        let group_repo: Arc<dyn GroupRepository> = Arc::new(
            storeit_db_sqlite::group_repo::SqliteGroupRepository::new(pool.clone()),
        );
        let user_group_repo: Arc<dyn UserGroupRepository> = Arc::new(
            storeit_db_sqlite::user_group_repo::SqliteUserGroupRepository::new(pool.clone()),
        );
        let settings_repo: Arc<dyn SettingsRepository> =
            Arc::new(storeit_db_sqlite::settings_repo::SqliteSettingsRepository::new(pool.clone()));

        // Determine effective image path:
        // env var > DB setting > platform default (already in config.image_storage_path)
        let env_image_path = std::env::var("STOREIT_IMAGE_PATH").is_ok();
        let effective_image_path = if env_image_path {
            // Env var was set — use config value (which comes from env)
            config.image_storage_path.clone()
        } else if let Some(db_path) = settings_repo.get("image_storage_path").await? {
            // DB has a persisted path — use it
            db_path
        } else {
            // Platform default (already resolved in config)
            config.image_storage_path.clone()
        };

        // Ensure the effective image directory exists
        let effective_dir = std::path::Path::new(&effective_image_path);
        if !effective_dir.exists() {
            std::fs::create_dir_all(effective_dir)?;
        }

        let state = Self {
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
                pool.clone(),
            )),
            settings_repo,
            image_storage: std::sync::RwLock::new(Arc::new(
                storeit_storage_fs::FsImageStorage::new(&effective_image_path),
            )),
            image_storage_path: std::sync::RwLock::new(effective_image_path),
            env_image_path,

            // AI
            item_identifier,

            // Auth
            user_repo,
            group_repo,
            user_group_repo,
            session_repo: Arc::new(
                storeit_db_sqlite::session_repo::SqliteSessionRepository::new(pool),
            ),
            auth_provider,
            auth_mode,
            session_secret: config.session_secret.clone(),
            session_ttl_hours: config.session_ttl_hours,

            backup_jobs: DashMap::new(),
            restore_jobs: DashMap::new(),
        };

        // Seed admin user in local auth mode
        if auth_mode == AuthMode::Local {
            seed_admin(&state, config).await?;
        }

        Ok(state)
    }
}

/// Seed the admin user if no admins exist yet (local auth mode only).
async fn seed_admin(state: &AppState, config: &crate::config::Config) -> anyhow::Result<()> {
    let admin_count = state.user_repo.count_admins().await?;
    if admin_count > 0 {
        tracing::debug!("Admin user(s) already exist, skipping seed");
        return Ok(());
    }

    if config.admin_password == "changeme" {
        tracing::warn!(
            "Creating admin user with default password 'changeme' — \
             set STOREIT_ADMIN_PASSWORD to a secure value!"
        );
    }

    let password_hash = storeit_auth::hash_password(&config.admin_password)
        .map_err(|e| anyhow::anyhow!("failed to hash admin password: {e}"))?;

    let user = state
        .user_repo
        .create_local(CreateLocalUser {
            username: config.admin_username.clone(),
            email: config.admin_email.clone(),
            display_name: config.admin_display_name.clone(),
            password_hash,
            is_admin: true,
        })
        .await?;

    // Add admin to the default group as Owner
    let default_group = state.group_repo.get_or_create_by_name("default").await?;
    state
        .user_group_repo
        .add_member(user.id, default_group.id, GroupRole::Owner)
        .await?;

    tracing::info!(
        "Seeded admin user '{}' ({})",
        config.admin_username,
        user.id
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- BackupJob tests --

    #[test]
    fn backup_job_new_defaults() {
        let job = BackupJob::new();
        assert_eq!(job.status(), "pending");
        assert_eq!(job.progress.load(Ordering::Relaxed), 0);
        assert_eq!(job.total.load(Ordering::Relaxed), 0);
        assert!(job.error.lock().unwrap().is_none());
        assert!(job.archive_path.lock().unwrap().is_none());
    }

    #[test]
    fn backup_job_set_status() {
        let job = BackupJob::new();
        job.set_status("running");
        assert_eq!(job.status(), "running");
    }

    #[test]
    fn backup_job_inc_progress() {
        let job = BackupJob::new();
        job.inc_progress();
        job.inc_progress();
        job.inc_progress();
        assert_eq!(job.progress.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn backup_job_set_error() {
        let job = BackupJob::new();
        job.set_error("disk full".into());
        assert_eq!(job.status(), "failed");
        assert_eq!(job.error.lock().unwrap().as_deref(), Some("disk full"));
    }

    #[test]
    fn backup_job_set_complete() {
        let job = BackupJob::new();
        job.set_complete(PathBuf::from("/tmp/backup.tar.gz"));
        assert_eq!(job.status(), "complete");
        assert_eq!(
            job.archive_path.lock().unwrap().as_deref(),
            Some(std::path::Path::new("/tmp/backup.tar.gz"))
        );
    }

    // -- RestoreJob tests --

    #[test]
    fn restore_job_new_defaults() {
        let job = RestoreJob::new();
        assert_eq!(job.status(), "pending");
        assert_eq!(job.progress.load(Ordering::Relaxed), 0);
        assert_eq!(job.total.load(Ordering::Relaxed), 0);
        assert!(job.error.lock().unwrap().is_none());
    }

    #[test]
    fn restore_job_set_status() {
        let job = RestoreJob::new();
        job.set_status("running");
        assert_eq!(job.status(), "running");
    }

    #[test]
    fn restore_job_inc_progress() {
        let job = RestoreJob::new();
        job.inc_progress();
        job.inc_progress();
        assert_eq!(job.progress.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn restore_job_set_error() {
        let job = RestoreJob::new();
        job.set_error("corrupt archive".into());
        assert_eq!(job.status(), "failed");
        assert_eq!(
            job.error.lock().unwrap().as_deref(),
            Some("corrupt archive")
        );
    }

    #[test]
    fn restore_job_set_complete() {
        let job = RestoreJob::new();
        job.set_complete();
        assert_eq!(job.status(), "complete");
    }

    #[tokio::test]
    async fn new_with_in_memory_db() {
        let config = crate::config::Config {
            bind_addr: "127.0.0.1:0".into(),
            database_url: "sqlite::memory:".into(),
            image_storage_path: "/tmp/storeit_test_images".into(),
            auth_issuer_url: None,
            auth_client_id: "test".into(),
            auth_client_secret: "test".into(),
            auth_redirect_uri: "http://localhost/callback".into(),
            auth_group_prefix: "storeit:".into(),
            session_secret: "test-secret-must-be-at-least-32-ch".into(),
            session_ttl_hours: 24,
            admin_username: "admin".into(),
            admin_password: "testpass".into(),
            admin_email: "admin@test.com".into(),
            admin_display_name: "Test Admin".into(),
            anthropic_api_key: None,
            ai_model: "claude-haiku-4-5-20251001".into(),
            claude_code_path: "claude".into(),
        };
        let _state = AppState::new(&config).await.unwrap();
    }

    #[tokio::test]
    async fn creates_data_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("nested").join("db").join("storeit.db");
        let img_dir = tmp.path().join("nested").join("images");

        // Neither directory exists yet
        assert!(!db_path.parent().unwrap().exists());
        assert!(!img_dir.exists());

        let config = crate::config::Config {
            bind_addr: "127.0.0.1:0".into(),
            database_url: format!("sqlite:{}?mode=rwc", db_path.display()),
            image_storage_path: img_dir.to_string_lossy().into_owned(),
            auth_issuer_url: None,
            auth_client_id: "test".into(),
            auth_client_secret: "test".into(),
            auth_redirect_uri: "http://localhost/callback".into(),
            auth_group_prefix: "storeit:".into(),
            session_secret: "test-secret-must-be-at-least-32-ch".into(),
            session_ttl_hours: 24,
            admin_username: "admin".into(),
            admin_password: "testpass".into(),
            admin_email: "admin@test.com".into(),
            admin_display_name: "Test Admin".into(),
            anthropic_api_key: None,
            ai_model: "claude-haiku-4-5-20251001".into(),
            claude_code_path: "claude".into(),
        };

        let _state = AppState::new(&config).await.unwrap();

        assert!(
            db_path.parent().unwrap().exists(),
            "db parent dir should be created"
        );
        assert!(db_path.exists(), "sqlite db file should be created");
        assert!(img_dir.exists(), "image dir should be created");
    }

    #[tokio::test]
    async fn settings_repo_loaded_in_app_state() {
        let tmp = tempfile::tempdir().unwrap();
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let db = storeit_db_sqlite::SqliteDb::new(pool.clone());
        db.migrate().await.unwrap();

        let settings_repo: Arc<dyn SettingsRepository> =
            Arc::new(storeit_db_sqlite::settings_repo::SqliteSettingsRepository::new(pool.clone()));

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
                pool.clone(),
            )),
            settings_repo: settings_repo.clone(),
            image_storage: std::sync::RwLock::new(Arc::new(
                storeit_storage_fs::FsImageStorage::new(tmp.path().to_str().unwrap()),
            )),
            image_storage_path: std::sync::RwLock::new(tmp.path().to_string_lossy().into_owned()),
            env_image_path: false,
            item_identifier: None,
            user_repo: Arc::new(storeit_db_sqlite::user_repo::SqliteUserRepository::new(
                pool.clone(),
            )),
            group_repo: Arc::new(storeit_db_sqlite::group_repo::SqliteGroupRepository::new(
                pool.clone(),
            )),
            user_group_repo: Arc::new(
                storeit_db_sqlite::user_group_repo::SqliteUserGroupRepository::new(pool.clone()),
            ),
            session_repo: Arc::new(
                storeit_db_sqlite::session_repo::SqliteSessionRepository::new(pool),
            ),
            auth_provider: None,
            auth_mode: AuthMode::Local,
            session_secret: "test-secret-must-be-at-least-32-ch".into(),
            session_ttl_hours: 24,
            backup_jobs: DashMap::new(),
            restore_jobs: DashMap::new(),
        };

        // Verify settings_repo is accessible
        state
            .settings_repo
            .set("test_key", "test_val")
            .await
            .unwrap();
        let val = state.settings_repo.get("test_key").await.unwrap();
        assert_eq!(val, Some("test_val".to_string()));
    }
}
