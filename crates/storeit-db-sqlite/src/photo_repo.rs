use async_trait::async_trait;
use sqlx::SqlitePool;
use storeit_domain::entities::*;
use storeit_domain::errors::{DomainError, Result};
use storeit_domain::repositories::PhotoRepository;
use uuid::Uuid;

pub struct SqlitePhotoRepository {
    pool: SqlitePool,
}

impl SqlitePhotoRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn parse_uuid(s: &str) -> Result<Uuid> {
    s.parse()
        .map_err(|e: uuid::Error| DomainError::Internal(e.to_string()))
}

fn parse_datetime(s: &str) -> Result<chrono::DateTime<chrono::Utc>> {
    let s = if s.ends_with('Z') || s.contains('+') {
        s.to_string()
    } else {
        format!("{s}Z")
    };
    s.parse()
        .map_err(|e: chrono::ParseError| DomainError::Internal(e.to_string()))
}

#[async_trait]
impl PhotoRepository for SqlitePhotoRepository {
    async fn create(
        &self,
        input: CreatePhoto,
        storage_key: String,
        thumbnail_key: Option<String>,
        large_key: Option<String>,
    ) -> Result<Photo> {
        let id = Uuid::new_v4();
        let id_str = id.to_string();
        let entity_type_str = input.entity_type.as_str();
        let entity_id_str = input.entity_id.to_string();

        sqlx::query!(
            r#"INSERT INTO photos (id, entity_type, entity_id, storage_key, thumbnail_key, large_key, mime_type, is_primary, rotation)
               VALUES (?, ?, ?, ?, ?, ?, ?, 0, 0)"#,
            id_str,
            entity_type_str,
            entity_id_str,
            storage_key,
            thumbnail_key,
            large_key,
            input.mime_type,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        self.get(id).await?.ok_or(DomainError::Internal(
            "failed to read back created photo".into(),
        ))
    }

    async fn get(&self, id: Uuid) -> Result<Option<Photo>> {
        let id_str = id.to_string();

        let row = sqlx::query!(
            r#"SELECT id, entity_type, entity_id, storage_key, thumbnail_key, large_key, mime_type, is_primary, rotation, created_at
               FROM photos WHERE id = ?"#,
            id_str,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(Photo {
                id: parse_uuid(&r.id)?,
                entity_type: r.entity_type.parse()?,
                entity_id: parse_uuid(&r.entity_id)?,
                storage_key: r.storage_key,
                thumbnail_key: r.thumbnail_key,
                large_key: r.large_key,
                mime_type: r.mime_type,
                is_primary: r.is_primary != 0,
                rotation: r.rotation as i32,
                created_at: parse_datetime(&r.created_at)?,
            })),
            None => Ok(None),
        }
    }

    async fn list_by_entity(&self, entity_type: EntityType, entity_id: Uuid) -> Result<Vec<Photo>> {
        let et_str = entity_type.as_str();
        let eid_str = entity_id.to_string();

        let rows = sqlx::query!(
            r#"SELECT id, entity_type, entity_id, storage_key, thumbnail_key, large_key, mime_type, is_primary, rotation, created_at
               FROM photos WHERE entity_type = ? AND entity_id = ?
               ORDER BY is_primary DESC, created_at"#,
            et_str,
            eid_str,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                Ok(Photo {
                    id: parse_uuid(&r.id)?,
                    entity_type: r.entity_type.parse()?,
                    entity_id: parse_uuid(&r.entity_id)?,
                    storage_key: r.storage_key,
                    thumbnail_key: r.thumbnail_key,
                    large_key: r.large_key,
                    mime_type: r.mime_type,
                    is_primary: r.is_primary != 0,
                    rotation: r.rotation as i32,
                    created_at: parse_datetime(&r.created_at)?,
                })
            })
            .collect()
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let id_str = id.to_string();

        let result = sqlx::query!("DELETE FROM photos WHERE id = ?", id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound {
                entity_type: "photo".into(),
                id,
            });
        }

        Ok(())
    }

    async fn set_primary(&self, id: Uuid, entity_type: EntityType, entity_id: Uuid) -> Result<()> {
        let et_str = entity_type.as_str();
        let eid_str = entity_id.to_string();
        let id_str = id.to_string();

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        sqlx::query!(
            "UPDATE photos SET is_primary = 0 WHERE entity_type = ? AND entity_id = ?",
            et_str,
            eid_str,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        sqlx::query!("UPDATE photos SET is_primary = 1 WHERE id = ?", id_str,)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<Photo>> {
        let rows = sqlx::query!(
            r#"SELECT id, entity_type, entity_id, storage_key, thumbnail_key, large_key, mime_type, is_primary, rotation, created_at
               FROM photos ORDER BY created_at"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                Ok(Photo {
                    id: parse_uuid(&r.id)?,
                    entity_type: r.entity_type.parse()?,
                    entity_id: parse_uuid(&r.entity_id)?,
                    storage_key: r.storage_key,
                    thumbnail_key: r.thumbnail_key,
                    large_key: r.large_key,
                    mime_type: r.mime_type,
                    is_primary: r.is_primary != 0,
                    rotation: r.rotation as i32,
                    created_at: parse_datetime(&r.created_at)?,
                })
            })
            .collect()
    }

    async fn count_by_storage_key(&self, storage_key: &str) -> Result<i64> {
        let row = sqlx::query_scalar!(
            "SELECT COUNT(*) as count FROM photos WHERE storage_key = ?",
            storage_key,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row as i64)
    }

    async fn update_storage_key(&self, id: Uuid, new_key: &str) -> Result<()> {
        let id_str = id.to_string();

        let result = sqlx::query!(
            "UPDATE photos SET storage_key = ? WHERE id = ?",
            new_key,
            id_str,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound {
                entity_type: "photo".into(),
                id,
            });
        }

        Ok(())
    }

    async fn set_rotation(&self, id: Uuid, rotation: i32) -> Result<()> {
        let id_str = id.to_string();

        let result = sqlx::query!(
            "UPDATE photos SET rotation = ? WHERE id = ?",
            rotation,
            id_str,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound {
                entity_type: "photo".into(),
                id,
            });
        }

        Ok(())
    }

    async fn insert_raw(&self, photo: &Photo) -> Result<()> {
        let id_str = photo.id.to_string();
        let et_str = photo.entity_type.as_str();
        let eid_str = photo.entity_id.to_string();
        let is_primary: i32 = if photo.is_primary { 1 } else { 0 };
        let created = photo.created_at.format("%Y-%m-%dT%H:%M:%S%.3f").to_string();

        sqlx::query!(
            r#"INSERT INTO photos (id, entity_type, entity_id, storage_key, thumbnail_key, large_key, mime_type, is_primary, rotation, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            id_str,
            et_str,
            eid_str,
            photo.storage_key,
            photo.thumbnail_key,
            photo.large_key,
            photo.mime_type,
            is_primary,
            photo.rotation,
            created,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete_all(&self) -> Result<()> {
        sqlx::query!("DELETE FROM photos")
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }
}
