use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use storeit_domain::entities::*;
use storeit_domain::errors::{DomainError, Result};
use storeit_domain::repositories::UserGroupRepository;
use uuid::Uuid;

pub struct SqliteUserGroupRepository {
    pool: SqlitePool,
}

impl SqliteUserGroupRepository {
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
impl UserGroupRepository for SqliteUserGroupRepository {
    async fn set_memberships(&self, user_id: Uuid, groups: Vec<(Uuid, GroupRole)>) -> Result<()> {
        let uid = user_id.to_string();

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        sqlx::query("DELETE FROM user_groups WHERE user_id = ?1")
            .bind(&uid)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        for (group_id, role) in &groups {
            let gid = group_id.to_string();
            let role_str = role.as_str();
            sqlx::query(r#"INSERT INTO user_groups (user_id, group_id, role) VALUES (?1, ?2, ?3)"#)
                .bind(&uid)
                .bind(&gid)
                .bind(role_str)
                .execute(&mut *tx)
                .await
                .map_err(|e| DomainError::Database(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list_groups_for_user(&self, user_id: Uuid) -> Result<Vec<(Group, GroupRole)>> {
        let uid = user_id.to_string();

        let rows = sqlx::query(
            r#"SELECT g.id, g.name, g.created_at, g.updated_at, ug.role
               FROM groups g
               JOIN user_groups ug ON g.id = ug.group_id
               WHERE ug.user_id = ?1
               ORDER BY g.name"#,
        )
        .bind(&uid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.iter()
            .map(|r| {
                let group = Group {
                    id: parse_uuid(r.get::<&str, _>("id"))?,
                    name: r.get::<String, _>("name"),
                    created_at: parse_datetime(r.get::<&str, _>("created_at"))?,
                    updated_at: parse_datetime(r.get::<&str, _>("updated_at"))?,
                };
                let role: GroupRole = r
                    .get::<String, _>("role")
                    .parse()
                    .map_err(|_| DomainError::Internal("invalid role in db".into()))?;
                Ok((group, role))
            })
            .collect()
    }

    async fn is_member(&self, user_id: Uuid, group_id: Uuid) -> Result<bool> {
        let uid = user_id.to_string();
        let gid = group_id.to_string();

        let row = sqlx::query(
            r#"SELECT COUNT(*) as count FROM user_groups WHERE user_id = ?1 AND group_id = ?2"#,
        )
        .bind(&uid)
        .bind(&gid)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row.get::<i32, _>("count") > 0)
    }

    async fn add_member(&self, user_id: Uuid, group_id: Uuid, role: GroupRole) -> Result<()> {
        let uid = user_id.to_string();
        let gid = group_id.to_string();
        let role_str = role.as_str();

        sqlx::query(
            r#"INSERT OR IGNORE INTO user_groups (user_id, group_id, role) VALUES (?1, ?2, ?3)"#,
        )
        .bind(&uid)
        .bind(&gid)
        .bind(role_str)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn remove_member(&self, user_id: Uuid, group_id: Uuid) -> Result<()> {
        let uid = user_id.to_string();
        let gid = group_id.to_string();

        sqlx::query(r#"DELETE FROM user_groups WHERE user_id = ?1 AND group_id = ?2"#)
            .bind(&uid)
            .bind(&gid)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list_members_of_group(&self, group_id: Uuid) -> Result<Vec<(User, GroupRole)>> {
        let gid = group_id.to_string();

        let rows = sqlx::query(
            r#"SELECT u.id, u.external_id, u.email, u.display_name, u.is_admin,
                      u.created_at, u.updated_at, ug.role
               FROM users u
               JOIN user_groups ug ON u.id = ug.user_id
               WHERE ug.group_id = ?1
               ORDER BY u.display_name"#,
        )
        .bind(&gid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.iter()
            .map(|r| {
                let user = User {
                    id: parse_uuid(r.get::<&str, _>("id"))?,
                    external_id: r.get::<String, _>("external_id"),
                    email: r.get::<String, _>("email"),
                    display_name: r.get::<String, _>("display_name"),
                    is_admin: r.get::<i32, _>("is_admin") != 0,
                    created_at: parse_datetime(r.get::<&str, _>("created_at"))?,
                    updated_at: parse_datetime(r.get::<&str, _>("updated_at"))?,
                };
                let role: GroupRole = r
                    .get::<String, _>("role")
                    .parse()
                    .map_err(|_| DomainError::Internal("invalid role in db".into()))?;
                Ok((user, role))
            })
            .collect()
    }

    async fn list_all(&self) -> Result<Vec<UserGroup>> {
        let rows =
            sqlx::query(r#"SELECT user_id, group_id, role FROM user_groups ORDER BY user_id"#)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.iter()
            .map(|r| {
                Ok(UserGroup {
                    user_id: parse_uuid(r.get::<&str, _>("user_id"))?,
                    group_id: parse_uuid(r.get::<&str, _>("group_id"))?,
                    role: r
                        .get::<String, _>("role")
                        .parse()
                        .map_err(|_| DomainError::Internal("invalid role in db".into()))?,
                })
            })
            .collect()
    }

    async fn insert_raw(&self, user_id: Uuid, group_id: Uuid, role: GroupRole) -> Result<()> {
        let uid = user_id.to_string();
        let gid = group_id.to_string();
        let role_str = role.as_str();

        sqlx::query(r#"INSERT INTO user_groups (user_id, group_id, role) VALUES (?1, ?2, ?3)"#)
            .bind(&uid)
            .bind(&gid)
            .bind(role_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete_all(&self) -> Result<()> {
        sqlx::query("DELETE FROM user_groups")
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }
}
