use sqlx::SqlitePool;

pub mod container_repo;
pub mod group_repo;
pub mod item_repo;
pub mod location_repo;
pub mod nfc_tag_repo;
pub mod photo_repo;
pub mod search_repo;
pub mod session_repo;
pub mod settings_repo;
pub mod user_group_repo;
pub mod user_repo;

/// Shared database pool wrapper.
#[derive(Clone)]
pub struct SqliteDb {
    pool: SqlitePool,
}

impl SqliteDb {
    /// The schema version that this binary expects.
    pub const EXPECTED_SCHEMA_VERSION: i64 = 1;

    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Run all embedded migrations.
    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("./migrations").run(&self.pool).await
    }

    /// Read the current schema version from the `_meta` table.
    /// Returns 0 if the table doesn't exist (pre-migration DB).
    pub async fn schema_version(&self) -> i64 {
        let result: Result<Option<(String,)>, _> =
            sqlx::query_as("SELECT value FROM _meta WHERE key = 'schema_version'")
                .fetch_optional(&self.pool)
                .await;
        match result {
            Ok(Some((v,))) => v.parse().unwrap_or(0),
            _ => 0,
        }
    }

    /// Set the schema version in the `_meta` table.
    pub async fn set_schema_version(&self, version: i64) {
        let _ =
            sqlx::query("INSERT OR REPLACE INTO _meta (key, value) VALUES ('schema_version', ?)")
                .bind(version.to_string())
                .execute(&self.pool)
                .await;
    }
}
