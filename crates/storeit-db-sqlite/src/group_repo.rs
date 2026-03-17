use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use storeit_domain::entities::*;
use storeit_domain::errors::{DomainError, Result};
use storeit_domain::repositories::GroupRepository;
use uuid::Uuid;

pub struct SqliteGroupRepository {
    pool: SqlitePool,
}

impl SqliteGroupRepository {
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

fn row_to_group(r: &sqlx::sqlite::SqliteRow) -> Result<Group> {
    Ok(Group {
        id: parse_uuid(r.get::<&str, _>("id"))?,
        name: r.get::<String, _>("name"),
        created_at: parse_datetime(r.get::<&str, _>("created_at"))?,
        updated_at: parse_datetime(r.get::<&str, _>("updated_at"))?,
    })
}

#[async_trait]
impl GroupRepository for SqliteGroupRepository {
    async fn get_or_create_by_name(&self, name: &str) -> Result<Group> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3f")
            .to_string();

        sqlx::query(
            r#"INSERT OR IGNORE INTO groups (id, name, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?3)"#,
        )
        .bind(&id)
        .bind(name)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        let row =
            sqlx::query(r#"SELECT id, name, created_at, updated_at FROM groups WHERE name = ?1"#)
                .bind(name)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| DomainError::Database(e.to_string()))?;

        row_to_group(&row)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Group>> {
        let id_str = id.to_string();
        let row =
            sqlx::query(r#"SELECT id, name, created_at, updated_at FROM groups WHERE id = ?1"#)
                .bind(&id_str)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| DomainError::Database(e.to_string()))?;

        row.as_ref().map(row_to_group).transpose()
    }

    async fn list_all(&self) -> Result<Vec<Group>> {
        let rows =
            sqlx::query(r#"SELECT id, name, created_at, updated_at FROM groups ORDER BY name"#)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.iter().map(row_to_group).collect()
    }

    async fn create(&self, name: &str) -> Result<Group> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3f")
            .to_string();

        sqlx::query(
            r#"INSERT INTO groups (id, name, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?3)"#,
        )
        .bind(&id)
        .bind(name)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        let row =
            sqlx::query(r#"SELECT id, name, created_at, updated_at FROM groups WHERE id = ?1"#)
                .bind(&id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| DomainError::Database(e.to_string()))?;

        row_to_group(&row)
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let id_str = id.to_string();

        // Check if group has members
        let member_count: i32 =
            sqlx::query(r#"SELECT COUNT(*) as count FROM user_groups WHERE group_id = ?1"#)
                .bind(&id_str)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| DomainError::Database(e.to_string()))?
                .get("count");

        if member_count > 0 {
            return Err(DomainError::Validation(
                "cannot delete group with members".into(),
            ));
        }

        // Refuse to delete the default group
        let default_group_id = "00000000-0000-0000-0000-000000000001";
        if id_str == default_group_id {
            return Err(DomainError::Validation(
                "cannot delete the default group".into(),
            ));
        }

        sqlx::query(r#"DELETE FROM groups WHERE id = ?1"#)
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn insert_raw(&self, group: &Group) -> Result<()> {
        let id_str = group.id.to_string();
        let created = group.created_at.format("%Y-%m-%dT%H:%M:%S%.3f").to_string();
        let updated = group.updated_at.format("%Y-%m-%dT%H:%M:%S%.3f").to_string();

        sqlx::query(
            r#"INSERT INTO groups (id, name, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4)"#,
        )
        .bind(&id_str)
        .bind(&group.name)
        .bind(&created)
        .bind(&updated)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete_all(&self) -> Result<()> {
        sqlx::query("DELETE FROM groups")
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }
}
