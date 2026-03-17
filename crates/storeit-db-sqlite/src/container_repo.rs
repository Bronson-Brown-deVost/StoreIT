use async_trait::async_trait;
use sqlx::SqlitePool;
use storeit_domain::entities::*;
use storeit_domain::errors::{DomainError, Result};
use storeit_domain::repositories::ContainerRepository;
use uuid::Uuid;

pub struct SqliteContainerRepository {
    pool: SqlitePool,
}

impl SqliteContainerRepository {
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

#[allow(clippy::too_many_arguments)]
fn row_to_container(
    id: &str,
    group_id: &str,
    parent_location_id: Option<&str>,
    parent_container_id: Option<&str>,
    name: String,
    description: Option<String>,
    color: Option<String>,
    created_at: &str,
    updated_at: &str,
) -> Result<Container> {
    Ok(Container {
        id: parse_uuid(id)?,
        group_id: parse_uuid(group_id)?,
        parent_location_id: parent_location_id.map(parse_uuid).transpose()?,
        parent_container_id: parent_container_id.map(parse_uuid).transpose()?,
        name,
        description,
        color,
        created_at: parse_datetime(created_at)?,
        updated_at: parse_datetime(updated_at)?,
    })
}

#[async_trait]
impl ContainerRepository for SqliteContainerRepository {
    async fn create(&self, group_id: Uuid, input: CreateContainer) -> Result<Container> {
        let id = Uuid::new_v4();
        let id_str = id.to_string();
        let group_str = group_id.to_string();
        let (parent_loc, parent_cont) = match &input.parent {
            ParentRef::Location(loc_id) => (Some(loc_id.to_string()), None),
            ParentRef::Container(cont_id) => (None, Some(cont_id.to_string())),
        };

        sqlx::query!(
            r#"INSERT INTO containers (id, group_id, parent_location_id, parent_container_id, name, description, color)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#,
            id_str,
            group_str,
            parent_loc,
            parent_cont,
            input.name,
            input.description,
            input.color,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        self.get(id, group_id).await?.ok_or(DomainError::Internal(
            "failed to read back created container".into(),
        ))
    }

    async fn get(&self, id: Uuid, group_id: Uuid) -> Result<Option<Container>> {
        let id_str = id.to_string();
        let group_str = group_id.to_string();

        let row = sqlx::query!(
            r#"SELECT id, group_id, parent_location_id, parent_container_id, name, description, color, created_at, updated_at
               FROM containers WHERE id = ? AND group_id = ?"#,
            id_str,
            group_str,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(row_to_container(
                &r.id,
                &r.group_id,
                r.parent_location_id.as_deref(),
                r.parent_container_id.as_deref(),
                r.name,
                r.description,
                r.color,
                &r.created_at,
                &r.updated_at,
            )?)),
            None => Ok(None),
        }
    }

    async fn update(&self, id: Uuid, group_id: Uuid, input: UpdateContainer) -> Result<Container> {
        let existing = self.get(id, group_id).await?.ok_or(DomainError::NotFound {
            entity_type: "container".into(),
            id,
        })?;

        let name = input.name.unwrap_or(existing.name);
        let description = input.description.or(existing.description);
        let color = input.color.or(existing.color);
        let id_str = id.to_string();
        let group_str = group_id.to_string();

        sqlx::query!(
            r#"UPDATE containers SET name = ?, description = ?, color = ?,
               updated_at = strftime('%Y-%m-%dT%H:%M:%f', 'now')
               WHERE id = ? AND group_id = ?"#,
            name,
            description,
            color,
            id_str,
            group_str,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        self.get(id, group_id).await?.ok_or(DomainError::Internal(
            "failed to read back updated container".into(),
        ))
    }

    async fn delete(&self, id: Uuid, group_id: Uuid) -> Result<()> {
        self.get(id, group_id).await?.ok_or(DomainError::NotFound {
            entity_type: "container".into(),
            id,
        })?;

        if self.has_children(id, group_id).await? {
            return Err(DomainError::NotEmpty {
                entity_type: "container".into(),
                id,
                child_count: 1,
            });
        }

        let id_str = id.to_string();
        let group_str = group_id.to_string();

        sqlx::query!(
            "DELETE FROM containers WHERE id = ? AND group_id = ?",
            id_str,
            group_str,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn move_to(&self, id: Uuid, group_id: Uuid, target: MoveTarget) -> Result<Container> {
        self.get(id, group_id).await?.ok_or(DomainError::NotFound {
            entity_type: "container".into(),
            id,
        })?;

        let id_str = id.to_string();
        let group_str = group_id.to_string();
        let (parent_loc, parent_cont) = match &target.target {
            ParentRef::Location(loc_id) => (Some(loc_id.to_string()), None),
            ParentRef::Container(cont_id) => (None, Some(cont_id.to_string())),
        };

        sqlx::query!(
            r#"UPDATE containers SET parent_location_id = ?, parent_container_id = ?,
               updated_at = strftime('%Y-%m-%dT%H:%M:%f', 'now')
               WHERE id = ? AND group_id = ?"#,
            parent_loc,
            parent_cont,
            id_str,
            group_str,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        self.get(id, group_id).await?.ok_or(DomainError::Internal(
            "failed to read back moved container".into(),
        ))
    }

    async fn list_by_location(&self, location_id: Uuid, group_id: Uuid) -> Result<Vec<Container>> {
        let loc_str = location_id.to_string();
        let group_str = group_id.to_string();

        let rows = sqlx::query!(
            r#"SELECT id, group_id, parent_location_id, parent_container_id, name, description, color, created_at, updated_at
               FROM containers WHERE parent_location_id = ? AND group_id = ?
               ORDER BY name"#,
            loc_str,
            group_str,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                row_to_container(
                    &r.id,
                    &r.group_id,
                    r.parent_location_id.as_deref(),
                    r.parent_container_id.as_deref(),
                    r.name,
                    r.description,
                    r.color,
                    &r.created_at,
                    &r.updated_at,
                )
            })
            .collect()
    }

    async fn list_by_container(
        &self,
        container_id: Uuid,
        group_id: Uuid,
    ) -> Result<Vec<Container>> {
        let cont_str = container_id.to_string();
        let group_str = group_id.to_string();

        let rows = sqlx::query!(
            r#"SELECT id, group_id, parent_location_id, parent_container_id, name, description, color, created_at, updated_at
               FROM containers WHERE parent_container_id = ? AND group_id = ?
               ORDER BY name"#,
            cont_str,
            group_str,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                row_to_container(
                    &r.id,
                    &r.group_id,
                    r.parent_location_id.as_deref(),
                    r.parent_container_id.as_deref(),
                    r.name,
                    r.description,
                    r.color,
                    &r.created_at,
                    &r.updated_at,
                )
            })
            .collect()
    }

    async fn list_all(&self, group_id: Uuid) -> Result<Vec<Container>> {
        let group_str = group_id.to_string();

        let rows = sqlx::query!(
            r#"SELECT id, group_id, parent_location_id, parent_container_id, name, description, color, created_at, updated_at
               FROM containers WHERE group_id = ?
               ORDER BY name"#,
            group_str,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                row_to_container(
                    &r.id,
                    &r.group_id,
                    r.parent_location_id.as_deref(),
                    r.parent_container_id.as_deref(),
                    r.name,
                    r.description,
                    r.color,
                    &r.created_at,
                    &r.updated_at,
                )
            })
            .collect()
    }

    async fn get_ancestry(&self, id: Uuid, group_id: Uuid) -> Result<Vec<AncestryNode>> {
        let id_str = id.to_string();
        let group_str = group_id.to_string();

        // Recursive CTE to walk up the parent chain
        let rows = sqlx::query(
            r#"WITH RECURSIVE path(id, name, parent_location_id, parent_container_id, entity_type, depth) AS (
                SELECT id, name, parent_location_id, parent_container_id, 'container', 0
                FROM containers WHERE id = ?1 AND group_id = ?2
                UNION ALL
                SELECT c.id, c.name, c.parent_location_id, c.parent_container_id, 'container', p.depth + 1
                FROM containers c
                INNER JOIN path p ON c.id = p.parent_container_id
                WHERE c.group_id = ?2
            )
            SELECT entity_type, id, name, parent_location_id FROM path ORDER BY depth DESC"#,
        )
        .bind(&id_str)
        .bind(&group_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        let mut ancestry = Vec::new();

        // The last row's parent_location_id tells us the root location
        if let Some(first_row) = rows.first() {
            use sqlx::Row;
            if let Some(loc_id_str) = first_row.get::<Option<String>, _>("parent_location_id") {
                // Fetch the location name
                let loc_row = sqlx::query!(
                    "SELECT id, name FROM locations WHERE id = ? AND group_id = ?",
                    loc_id_str,
                    group_str,
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| DomainError::Database(e.to_string()))?;

                if let Some(loc) = loc_row {
                    ancestry.push(AncestryNode {
                        entity_type: EntityType::Location,
                        id: parse_uuid(&loc.id)?,
                        name: loc.name,
                    });
                }
            }
        }

        // Add container nodes
        for row in &rows {
            use sqlx::Row;
            let rid: String = row.get("id");
            let rname: String = row.get("name");
            ancestry.push(AncestryNode {
                entity_type: EntityType::Container,
                id: parse_uuid(&rid)?,
                name: rname,
            });
        }

        Ok(ancestry)
    }

    async fn has_children(&self, id: Uuid, group_id: Uuid) -> Result<bool> {
        let id_str = id.to_string();
        let group_str = group_id.to_string();

        // Check nested containers
        let child_cont: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM containers WHERE parent_container_id = ? AND group_id = ?",
            id_str,
            group_str,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if child_cont > 0 {
            return Ok(true);
        }

        // Check items in this container
        let item_count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM items WHERE container_id = ? AND group_id = ?",
            id_str,
            group_str,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(item_count > 0)
    }

    async fn is_ancestor_of(
        &self,
        ancestor_id: Uuid,
        descendant_id: Uuid,
        group_id: Uuid,
    ) -> Result<bool> {
        let ancestor_str = ancestor_id.to_string();
        let descendant_str = descendant_id.to_string();
        let group_str = group_id.to_string();

        let result: Option<i32> = sqlx::query_scalar(
            r#"WITH RECURSIVE ancestors(id) AS (
                SELECT parent_container_id FROM containers WHERE id = ?1 AND group_id = ?3
                UNION ALL
                SELECT c.parent_container_id FROM containers c
                INNER JOIN ancestors a ON c.id = a.id
                WHERE c.parent_container_id IS NOT NULL AND c.group_id = ?3
            )
            SELECT 1 FROM ancestors WHERE id = ?2 LIMIT 1"#,
        )
        .bind(&descendant_str)
        .bind(&ancestor_str)
        .bind(&group_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(result.is_some())
    }

    async fn list_all_unscoped(&self) -> Result<Vec<Container>> {
        let rows = sqlx::query!(
            r#"SELECT id, group_id, parent_location_id, parent_container_id, name, description, color, created_at, updated_at
               FROM containers ORDER BY name"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                row_to_container(
                    &r.id,
                    &r.group_id,
                    r.parent_location_id.as_deref(),
                    r.parent_container_id.as_deref(),
                    r.name,
                    r.description,
                    r.color,
                    &r.created_at,
                    &r.updated_at,
                )
            })
            .collect()
    }

    async fn insert_raw(&self, container: &Container) -> Result<()> {
        let id_str = container.id.to_string();
        let group_str = container.group_id.to_string();
        let parent_loc = container.parent_location_id.map(|p| p.to_string());
        let parent_cont = container.parent_container_id.map(|p| p.to_string());
        let created = container
            .created_at
            .format("%Y-%m-%dT%H:%M:%S%.3f")
            .to_string();
        let updated = container
            .updated_at
            .format("%Y-%m-%dT%H:%M:%S%.3f")
            .to_string();

        sqlx::query!(
            r#"INSERT INTO containers (id, group_id, parent_location_id, parent_container_id, name, description, color, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            id_str,
            group_str,
            parent_loc,
            parent_cont,
            container.name,
            container.description,
            container.color,
            created,
            updated,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete_all(&self) -> Result<()> {
        sqlx::query!("DELETE FROM containers")
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }
}
