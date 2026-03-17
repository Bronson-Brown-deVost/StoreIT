use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use storeit_domain::entities::*;
use storeit_domain::errors::{DomainError, Result};
use storeit_domain::repositories::UserRepository;
use uuid::Uuid;

pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
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

fn row_to_user(r: &sqlx::sqlite::SqliteRow) -> Result<User> {
    Ok(User {
        id: parse_uuid(r.get::<&str, _>("id"))?,
        external_id: r.get::<String, _>("external_id"),
        email: r.get::<String, _>("email"),
        display_name: r.get::<String, _>("display_name"),
        is_admin: r.get::<i32, _>("is_admin") != 0,
        created_at: parse_datetime(r.get::<&str, _>("created_at"))?,
        updated_at: parse_datetime(r.get::<&str, _>("updated_at"))?,
    })
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn upsert_by_external_id(&self, input: CreateUser) -> Result<User> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3f")
            .to_string();

        sqlx::query(
            r#"INSERT INTO users (id, external_id, email, display_name, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?5)
               ON CONFLICT(external_id) DO UPDATE SET
                   email = excluded.email,
                   display_name = excluded.display_name,
                   updated_at = excluded.updated_at"#,
        )
        .bind(&id)
        .bind(&input.external_id)
        .bind(&input.email)
        .bind(&input.display_name)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        self.get_by_external_id(&input.external_id)
            .await?
            .ok_or_else(|| DomainError::Internal("user upsert failed".into()))
    }

    async fn get(&self, id: Uuid) -> Result<Option<User>> {
        let id_str = id.to_string();
        let row = sqlx::query(
            r#"SELECT id, external_id, email, display_name, is_admin, created_at, updated_at
               FROM users WHERE id = ?1"#,
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        row.as_ref().map(row_to_user).transpose()
    }

    async fn get_by_external_id(&self, external_id: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            r#"SELECT id, external_id, email, display_name, is_admin, created_at, updated_at
               FROM users WHERE external_id = ?1"#,
        )
        .bind(external_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        row.as_ref().map(row_to_user).transpose()
    }

    async fn create_local(&self, input: CreateLocalUser) -> Result<User> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3f")
            .to_string();
        let external_id = format!("local:{}", input.username);
        let is_admin_int: i32 = if input.is_admin { 1 } else { 0 };

        sqlx::query(
            r#"INSERT INTO users (id, external_id, email, display_name, password_hash, is_admin, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)"#,
        )
        .bind(&id)
        .bind(&external_id)
        .bind(&input.email)
        .bind(&input.display_name)
        .bind(&input.password_hash)
        .bind(is_admin_int)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        self.get_by_external_id(&external_id)
            .await?
            .ok_or_else(|| DomainError::Internal("user creation failed".into()))
    }

    async fn get_password_hash(&self, external_id: &str) -> Result<Option<String>> {
        let row = sqlx::query(r#"SELECT password_hash FROM users WHERE external_id = ?1"#)
            .bind(external_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row.and_then(|r| r.get::<Option<String>, _>("password_hash")))
    }

    async fn set_password_hash(&self, id: Uuid, hash: &str) -> Result<()> {
        let id_str = id.to_string();
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3f")
            .to_string();

        sqlx::query(r#"UPDATE users SET password_hash = ?1, updated_at = ?2 WHERE id = ?3"#)
            .bind(hash)
            .bind(&now)
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn set_admin(&self, id: Uuid, is_admin: bool) -> Result<()> {
        let id_str = id.to_string();
        let now = chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3f")
            .to_string();
        let is_admin_int: i32 = if is_admin { 1 } else { 0 };

        sqlx::query(r#"UPDATE users SET is_admin = ?1, updated_at = ?2 WHERE id = ?3"#)
            .bind(is_admin_int)
            .bind(&now)
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<User>> {
        let rows = sqlx::query(
            r#"SELECT id, external_id, email, display_name, is_admin, created_at, updated_at
               FROM users ORDER BY display_name"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.iter().map(row_to_user).collect()
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        let id_str = id.to_string();

        // Guard against deleting the last admin
        let user = self.get(id).await?.ok_or_else(|| DomainError::NotFound {
            entity_type: "user".into(),
            id,
        })?;

        if user.is_admin {
            let admin_count = self.count_admins().await?;
            if admin_count <= 1 {
                return Err(DomainError::Validation(
                    "cannot delete the last admin user".into(),
                ));
            }
        }

        // Delete user_groups first
        sqlx::query(r#"DELETE FROM user_groups WHERE user_id = ?1"#)
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        // Delete sessions
        sqlx::query(r#"DELETE FROM sessions WHERE user_id = ?1"#)
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        // Delete user
        sqlx::query(r#"DELETE FROM users WHERE id = ?1"#)
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn count_admins(&self) -> Result<i64> {
        let row = sqlx::query(r#"SELECT COUNT(*) as count FROM users WHERE is_admin = 1"#)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row.get::<i32, _>("count") as i64)
    }

    async fn insert_raw(&self, user: &User, password_hash: Option<&str>) -> Result<()> {
        let id_str = user.id.to_string();
        let is_admin_int: i32 = if user.is_admin { 1 } else { 0 };
        let created = user.created_at.format("%Y-%m-%dT%H:%M:%S%.3f").to_string();
        let updated = user.updated_at.format("%Y-%m-%dT%H:%M:%S%.3f").to_string();

        sqlx::query(
            r#"INSERT INTO users (id, external_id, email, display_name, password_hash, is_admin, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
        )
        .bind(&id_str)
        .bind(&user.external_id)
        .bind(&user.email)
        .bind(&user.display_name)
        .bind(password_hash)
        .bind(is_admin_int)
        .bind(&created)
        .bind(&updated)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete_all(&self) -> Result<()> {
        sqlx::query("DELETE FROM users")
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }
}
