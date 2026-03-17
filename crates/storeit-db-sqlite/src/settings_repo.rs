use async_trait::async_trait;
use sqlx::SqlitePool;
use storeit_domain::errors::{DomainError, Result};
use storeit_domain::repositories::SettingsRepository;

pub struct SqliteSettingsRepository {
    pool: SqlitePool,
}

impl SqliteSettingsRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SettingsRepository for SqliteSettingsRepository {
    async fn get(&self, key: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row.map(|(v,)| v))
    }

    async fn set(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<(String, String)>> {
        let rows: Vec<(String, String)> =
            sqlx::query_as("SELECT key, value FROM settings ORDER BY key")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(rows)
    }

    async fn delete_all(&self) -> Result<()> {
        sqlx::query("DELETE FROM settings")
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqliteSettingsRepository {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect");
        let db = crate::SqliteDb::new(pool.clone());
        db.migrate().await.expect("migrate");
        SqliteSettingsRepository::new(pool)
    }

    #[tokio::test]
    async fn get_returns_none_for_missing_key() {
        let repo = setup().await;
        let result = repo.get("nonexistent").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn set_and_get_roundtrip() {
        let repo = setup().await;
        repo.set("key", "value").await.unwrap();
        let result = repo.get("key").await.unwrap();
        assert_eq!(result, Some("value".to_string()));
    }

    #[tokio::test]
    async fn set_overwrites_existing() {
        let repo = setup().await;
        repo.set("k", "v1").await.unwrap();
        repo.set("k", "v2").await.unwrap();
        let result = repo.get("k").await.unwrap();
        assert_eq!(result, Some("v2".to_string()));
    }
}
