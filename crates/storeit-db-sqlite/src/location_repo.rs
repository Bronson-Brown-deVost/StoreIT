use async_trait::async_trait;
use sqlx::SqlitePool;
use storeit_domain::entities::*;
use storeit_domain::errors::{DomainError, Result};
use storeit_domain::repositories::LocationRepository;
use uuid::Uuid;

pub struct SqliteLocationRepository {
    pool: SqlitePool,
}

impl SqliteLocationRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn parse_uuid(s: &str) -> Result<Uuid> {
    s.parse()
        .map_err(|e: uuid::Error| DomainError::Internal(e.to_string()))
}

fn parse_datetime(s: &str) -> Result<chrono::DateTime<chrono::Utc>> {
    // SQLite strftime produces "YYYY-MM-DDTHH:MM:SS.sss" without timezone.
    // We append "Z" if needed and parse.
    let s = if s.ends_with('Z') || s.contains('+') {
        s.to_string()
    } else {
        format!("{s}Z")
    };
    s.parse()
        .map_err(|e: chrono::ParseError| DomainError::Internal(e.to_string()))
}

#[allow(clippy::too_many_arguments)]
fn row_to_location(
    id: &str,
    group_id: &str,
    parent_id: Option<&str>,
    name: String,
    description: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    created_at: &str,
    updated_at: &str,
) -> Result<Location> {
    Ok(Location {
        id: parse_uuid(id)?,
        group_id: parse_uuid(group_id)?,
        parent_id: parent_id.map(parse_uuid).transpose()?,
        name,
        description,
        latitude,
        longitude,
        created_at: parse_datetime(created_at)?,
        updated_at: parse_datetime(updated_at)?,
    })
}

#[async_trait]
impl LocationRepository for SqliteLocationRepository {
    async fn create(&self, group_id: Uuid, input: CreateLocation) -> Result<Location> {
        let id = Uuid::new_v4();
        let id_str = id.to_string();
        let group_str = group_id.to_string();
        let parent_str = input.parent_id.map(|p| p.to_string());

        sqlx::query!(
            r#"INSERT INTO locations (id, group_id, parent_id, name, description, latitude, longitude)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#,
            id_str,
            group_str,
            parent_str,
            input.name,
            input.description,
            input.latitude,
            input.longitude,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        self.get(id, group_id).await?.ok_or(DomainError::Internal(
            "failed to read back created location".into(),
        ))
    }

    async fn get(&self, id: Uuid, group_id: Uuid) -> Result<Option<Location>> {
        let id_str = id.to_string();
        let group_str = group_id.to_string();

        let row = sqlx::query!(
            r#"SELECT id, group_id, parent_id, name, description, latitude, longitude, created_at, updated_at
               FROM locations WHERE id = ? AND group_id = ?"#,
            id_str,
            group_str,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(row_to_location(
                &r.id,
                &r.group_id,
                r.parent_id.as_deref(),
                r.name,
                r.description,
                r.latitude,
                r.longitude,
                &r.created_at,
                &r.updated_at,
            )?)),
            None => Ok(None),
        }
    }

    async fn update(&self, id: Uuid, group_id: Uuid, input: UpdateLocation) -> Result<Location> {
        let existing = self.get(id, group_id).await?.ok_or(DomainError::NotFound {
            entity_type: "location".into(),
            id,
        })?;

        let name = input.name.unwrap_or(existing.name);
        let description = input.description.or(existing.description);
        let latitude = input.latitude.or(existing.latitude);
        let longitude = input.longitude.or(existing.longitude);
        let id_str = id.to_string();
        let group_str = group_id.to_string();

        sqlx::query!(
            r#"UPDATE locations SET name = ?, description = ?, latitude = ?, longitude = ?,
               updated_at = strftime('%Y-%m-%dT%H:%M:%f', 'now')
               WHERE id = ? AND group_id = ?"#,
            name,
            description,
            latitude,
            longitude,
            id_str,
            group_str,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        self.get(id, group_id).await?.ok_or(DomainError::Internal(
            "failed to read back updated location".into(),
        ))
    }

    async fn delete(&self, id: Uuid, group_id: Uuid) -> Result<()> {
        // Check existence
        self.get(id, group_id).await?.ok_or(DomainError::NotFound {
            entity_type: "location".into(),
            id,
        })?;

        // Check for children (child locations or containers)
        if self.has_children(id, group_id).await? {
            return Err(DomainError::NotEmpty {
                entity_type: "location".into(),
                id,
                child_count: 1, // simplified
            });
        }

        let id_str = id.to_string();
        let group_str = group_id.to_string();

        sqlx::query!(
            "DELETE FROM locations WHERE id = ? AND group_id = ?",
            id_str,
            group_str,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list_roots(&self, group_id: Uuid) -> Result<Vec<Location>> {
        let group_str = group_id.to_string();

        let rows = sqlx::query!(
            r#"SELECT id, group_id, parent_id, name, description, latitude, longitude, created_at, updated_at
               FROM locations WHERE group_id = ? AND parent_id IS NULL
               ORDER BY name"#,
            group_str,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                row_to_location(
                    &r.id,
                    &r.group_id,
                    r.parent_id.as_deref(),
                    r.name,
                    r.description,
                    r.latitude,
                    r.longitude,
                    &r.created_at,
                    &r.updated_at,
                )
            })
            .collect()
    }

    async fn list_children(&self, parent_id: Uuid, group_id: Uuid) -> Result<Vec<Location>> {
        let parent_str = parent_id.to_string();
        let group_str = group_id.to_string();

        let rows = sqlx::query!(
            r#"SELECT id, group_id, parent_id, name, description, latitude, longitude, created_at, updated_at
               FROM locations WHERE group_id = ? AND parent_id = ?
               ORDER BY name"#,
            group_str,
            parent_str,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                row_to_location(
                    &r.id,
                    &r.group_id,
                    r.parent_id.as_deref(),
                    r.name,
                    r.description,
                    r.latitude,
                    r.longitude,
                    &r.created_at,
                    &r.updated_at,
                )
            })
            .collect()
    }

    async fn get_tree(&self, group_id: Uuid) -> Result<Vec<LocationTreeNode>> {
        let group_str = group_id.to_string();

        let rows = sqlx::query!(
            r#"SELECT id, group_id, parent_id, name, description, latitude, longitude, created_at, updated_at
               FROM locations WHERE group_id = ?
               ORDER BY name"#,
            group_str,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        let locations: Vec<Location> = rows
            .into_iter()
            .map(|r| {
                row_to_location(
                    &r.id,
                    &r.group_id,
                    r.parent_id.as_deref(),
                    r.name,
                    r.description,
                    r.latitude,
                    r.longitude,
                    &r.created_at,
                    &r.updated_at,
                )
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(build_tree(&locations, None))
    }

    async fn has_children(&self, id: Uuid, group_id: Uuid) -> Result<bool> {
        let id_str = id.to_string();
        let group_str = group_id.to_string();

        // Check child locations
        let child_loc_count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM locations WHERE parent_id = ? AND group_id = ?",
            id_str,
            group_str,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if child_loc_count > 0 {
            return Ok(true);
        }

        // Check containers at this location
        let container_count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM containers WHERE parent_location_id = ? AND group_id = ?",
            id_str,
            group_str,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if container_count > 0 {
            return Ok(true);
        }

        // Check items directly at this location
        let item_count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM items WHERE location_id = ? AND group_id = ?",
            id_str,
            group_str,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(item_count > 0)
    }

    async fn list_all_unscoped(&self) -> Result<Vec<Location>> {
        let rows = sqlx::query!(
            r#"SELECT id, group_id, parent_id, name, description, latitude, longitude, created_at, updated_at
               FROM locations ORDER BY name"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                row_to_location(
                    &r.id,
                    &r.group_id,
                    r.parent_id.as_deref(),
                    r.name,
                    r.description,
                    r.latitude,
                    r.longitude,
                    &r.created_at,
                    &r.updated_at,
                )
            })
            .collect()
    }

    async fn insert_raw(&self, location: &Location) -> Result<()> {
        let id_str = location.id.to_string();
        let group_str = location.group_id.to_string();
        let parent_str = location.parent_id.map(|p| p.to_string());
        let created = location
            .created_at
            .format("%Y-%m-%dT%H:%M:%S%.3f")
            .to_string();
        let updated = location
            .updated_at
            .format("%Y-%m-%dT%H:%M:%S%.3f")
            .to_string();

        sqlx::query!(
            r#"INSERT INTO locations (id, group_id, parent_id, name, description, latitude, longitude, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            id_str,
            group_str,
            parent_str,
            location.name,
            location.description,
            location.latitude,
            location.longitude,
            created,
            updated,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete_all(&self) -> Result<()> {
        sqlx::query!("DELETE FROM locations")
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }
}

fn build_tree(locations: &[Location], parent_id: Option<Uuid>) -> Vec<LocationTreeNode> {
    locations
        .iter()
        .filter(|l| l.parent_id == parent_id)
        .map(|l| LocationTreeNode {
            location: l.clone(),
            children: build_tree(locations, Some(l.id)),
        })
        .collect()
}
