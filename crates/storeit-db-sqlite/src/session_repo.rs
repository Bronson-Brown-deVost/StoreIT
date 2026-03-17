use async_trait::async_trait;
use sqlx::SqlitePool;
use storeit_domain::entities::*;
use storeit_domain::errors::{DomainError, Result};
use storeit_domain::repositories::SessionRepository;
use uuid::Uuid;

pub struct SqliteSessionRepository {
    pool: SqlitePool,
}

impl SqliteSessionRepository {
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
impl SessionRepository for SqliteSessionRepository {
    async fn create(&self, session: Session) -> Result<Session> {
        let uid = session.user_id.to_string();
        let gid = session.active_group_id.to_string();
        let expires = session
            .expires_at
            .format("%Y-%m-%dT%H:%M:%S%.3f")
            .to_string();
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3f")
            .to_string();

        sqlx::query!(
            r#"INSERT INTO sessions (id, user_id, active_group_id, expires_at, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5)"#,
            session.id,
            uid,
            gid,
            expires,
            now,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        self.get(&session.id)
            .await?
            .ok_or_else(|| DomainError::Internal("session create failed".into()))
    }

    async fn get(&self, id: &str) -> Result<Option<Session>> {
        let row = sqlx::query!(
            r#"SELECT id, user_id, active_group_id, expires_at, created_at
               FROM sessions WHERE id = ?1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        row.map(|r| {
            Ok(Session {
                id: r.id,
                user_id: parse_uuid(&r.user_id)?,
                active_group_id: parse_uuid(&r.active_group_id)?,
                expires_at: parse_datetime(&r.expires_at)?,
                created_at: parse_datetime(&r.created_at)?,
            })
        })
        .transpose()
    }

    async fn update_active_group(&self, id: &str, group_id: Uuid) -> Result<Session> {
        let gid = group_id.to_string();

        let result = sqlx::query!(
            r#"UPDATE sessions SET active_group_id = ?1 WHERE id = ?2"#,
            gid,
            id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::NotFound {
                entity_type: "session".into(),
                id: Uuid::nil(),
            });
        }

        self.get(id)
            .await?
            .ok_or_else(|| DomainError::Internal("session update failed".into()))
    }

    async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query!("DELETE FROM sessions WHERE id = ?1", id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete_expired(&self) -> Result<u64> {
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3f")
            .to_string();

        let result = sqlx::query!("DELETE FROM sessions WHERE expires_at < ?1", now)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(result.rows_affected())
    }

    async fn delete_all(&self) -> Result<()> {
        sqlx::query("DELETE FROM sessions")
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }
}
