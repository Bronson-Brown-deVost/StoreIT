use async_trait::async_trait;
use sqlx::SqlitePool;
use storeit_domain::entities::*;
use storeit_domain::errors::{DomainError, Result};
use storeit_domain::repositories::NfcTagRepository;
use uuid::Uuid;

pub struct SqliteNfcTagRepository {
    pool: SqlitePool,
}

impl SqliteNfcTagRepository {
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

fn row_to_nfc_tag(
    id: &str,
    group_id: &str,
    tag_uri: String,
    entity_type: Option<String>,
    entity_id: Option<String>,
    created_at: &str,
    assigned_at: Option<&str>,
) -> Result<NfcTag> {
    Ok(NfcTag {
        id: parse_uuid(id)?,
        group_id: parse_uuid(group_id)?,
        tag_uri,
        entity_type: entity_type.map(|s| s.parse()).transpose()?,
        entity_id: entity_id.as_deref().map(parse_uuid).transpose()?,
        created_at: parse_datetime(created_at)?,
        assigned_at: assigned_at.map(parse_datetime).transpose()?,
    })
}

#[async_trait]
impl NfcTagRepository for SqliteNfcTagRepository {
    async fn get(&self, id: Uuid) -> Result<Option<NfcTag>> {
        let id_str = id.to_string();
        let row = sqlx::query!(
            r#"SELECT id, group_id, tag_uri, entity_type, entity_id, created_at, assigned_at
               FROM nfc_tags WHERE id = ?"#,
            id_str,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(row_to_nfc_tag(
                &r.id,
                &r.group_id,
                r.tag_uri,
                r.entity_type,
                r.entity_id,
                &r.created_at,
                r.assigned_at.as_deref(),
            )?)),
            None => Ok(None),
        }
    }

    async fn list_by_group(&self, group_id: Uuid) -> Result<Vec<NfcTag>> {
        let gid = group_id.to_string();
        let rows = sqlx::query!(
            r#"SELECT id, group_id, tag_uri, entity_type, entity_id, created_at, assigned_at
               FROM nfc_tags WHERE group_id = ? ORDER BY created_at DESC"#,
            gid,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                row_to_nfc_tag(
                    &r.id,
                    &r.group_id,
                    r.tag_uri,
                    r.entity_type,
                    r.entity_id,
                    &r.created_at,
                    r.assigned_at.as_deref(),
                )
            })
            .collect()
    }

    async fn list_by_entity(
        &self,
        entity_type: EntityType,
        entity_id: Uuid,
    ) -> Result<Vec<NfcTag>> {
        let et = entity_type.as_str();
        let eid = entity_id.to_string();
        let rows = sqlx::query!(
            r#"SELECT id, group_id, tag_uri, entity_type, entity_id, created_at, assigned_at
               FROM nfc_tags WHERE entity_type = ? AND entity_id = ?"#,
            et,
            eid,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                row_to_nfc_tag(
                    &r.id,
                    &r.group_id,
                    r.tag_uri,
                    r.entity_type,
                    r.entity_id,
                    &r.created_at,
                    r.assigned_at.as_deref(),
                )
            })
            .collect()
    }

    async fn create(&self, group_id: Uuid, tag_uri: String) -> Result<NfcTag> {
        let id = Uuid::new_v4();
        let id_str = id.to_string();
        let group_str = group_id.to_string();

        sqlx::query!(
            "INSERT INTO nfc_tags (id, group_id, tag_uri) VALUES (?, ?, ?)",
            id_str,
            group_str,
            tag_uri,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        self.get_by_uri(&tag_uri)
            .await?
            .ok_or(DomainError::Internal(
                "failed to read back created nfc tag".into(),
            ))
    }

    async fn get_by_uri(&self, tag_uri: &str) -> Result<Option<NfcTag>> {
        let row = sqlx::query!(
            r#"SELECT id, group_id, tag_uri, entity_type, entity_id, created_at, assigned_at
               FROM nfc_tags WHERE tag_uri = ?"#,
            tag_uri,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(row_to_nfc_tag(
                &r.id,
                &r.group_id,
                r.tag_uri,
                r.entity_type,
                r.entity_id,
                &r.created_at,
                r.assigned_at.as_deref(),
            )?)),
            None => Ok(None),
        }
    }

    async fn assign(&self, id: Uuid, entity_type: EntityType, entity_id: Uuid) -> Result<NfcTag> {
        let id_str = id.to_string();
        let et_str = entity_type.as_str();
        let eid_str = entity_id.to_string();

        let result = sqlx::query!(
            r#"UPDATE nfc_tags SET entity_type = ?, entity_id = ?,
               assigned_at = strftime('%Y-%m-%dT%H:%M:%f', 'now')
               WHERE id = ?"#,
            et_str,
            eid_str,
            id_str,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound {
                entity_type: "nfc_tag".into(),
                id,
            });
        }

        // Fetch by id
        let row = sqlx::query!(
            r#"SELECT id, group_id, tag_uri, entity_type, entity_id, created_at, assigned_at
               FROM nfc_tags WHERE id = ?"#,
            id_str,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some(r) => row_to_nfc_tag(
                &r.id,
                &r.group_id,
                r.tag_uri,
                r.entity_type,
                r.entity_id,
                &r.created_at,
                r.assigned_at.as_deref(),
            ),
            None => Err(DomainError::Internal(
                "failed to read back assigned nfc tag".into(),
            )),
        }
    }

    async fn unassign(&self, id: Uuid) -> Result<()> {
        let id_str = id.to_string();

        let result = sqlx::query!(
            "UPDATE nfc_tags SET entity_type = NULL, entity_id = NULL, assigned_at = NULL WHERE id = ?",
            id_str,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound {
                entity_type: "nfc_tag".into(),
                id,
            });
        }

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let id_str = id.to_string();

        let result = sqlx::query!("DELETE FROM nfc_tags WHERE id = ?", id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound {
                entity_type: "nfc_tag".into(),
                id,
            });
        }

        Ok(())
    }

    async fn list_all_unscoped(&self) -> Result<Vec<NfcTag>> {
        let rows = sqlx::query!(
            r#"SELECT id, group_id, tag_uri, entity_type, entity_id, created_at, assigned_at
               FROM nfc_tags ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                row_to_nfc_tag(
                    &r.id,
                    &r.group_id,
                    r.tag_uri,
                    r.entity_type,
                    r.entity_id,
                    &r.created_at,
                    r.assigned_at.as_deref(),
                )
            })
            .collect()
    }

    async fn insert_raw(&self, tag: &NfcTag) -> Result<()> {
        let id_str = tag.id.to_string();
        let group_str = tag.group_id.to_string();
        let et_str = tag.entity_type.map(|et| et.as_str().to_string());
        let eid_str = tag.entity_id.map(|id| id.to_string());
        let created = tag.created_at.format("%Y-%m-%dT%H:%M:%S%.3f").to_string();
        let assigned = tag
            .assigned_at
            .map(|a| a.format("%Y-%m-%dT%H:%M:%S%.3f").to_string());

        sqlx::query!(
            r#"INSERT INTO nfc_tags (id, group_id, tag_uri, entity_type, entity_id, created_at, assigned_at)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#,
            id_str,
            group_str,
            tag.tag_uri,
            et_str,
            eid_str,
            created,
            assigned,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete_all(&self) -> Result<()> {
        sqlx::query!("DELETE FROM nfc_tags")
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }
}
