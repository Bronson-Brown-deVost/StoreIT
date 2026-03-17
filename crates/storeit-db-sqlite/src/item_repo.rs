use async_trait::async_trait;
use sqlx::SqlitePool;
use storeit_domain::entities::*;
use storeit_domain::errors::{DomainError, Result};
use storeit_domain::repositories::ItemRepository;
use uuid::Uuid;

pub struct SqliteItemRepository {
    pool: SqlitePool,
}

impl SqliteItemRepository {
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
fn row_to_item(
    id: &str,
    group_id: &str,
    container_id: Option<&str>,
    location_id: Option<&str>,
    name: String,
    description: Option<String>,
    aliases_json: &str,
    keywords_json: &str,
    category: Option<String>,
    barcode: Option<String>,
    material: Option<String>,
    color: Option<String>,
    condition_notes: Option<String>,
    quantity: i64,
    ai_raw: Option<&str>,
    created_at: &str,
    updated_at: &str,
) -> Result<Item> {
    let aliases: Vec<String> = serde_json::from_str(aliases_json).unwrap_or_default();
    let keywords: Vec<String> = serde_json::from_str(keywords_json).unwrap_or_default();
    let ai_raw_val: Option<serde_json::Value> = ai_raw.and_then(|s| serde_json::from_str(s).ok());

    Ok(Item {
        id: parse_uuid(id)?,
        group_id: parse_uuid(group_id)?,
        container_id: container_id.map(parse_uuid).transpose()?,
        location_id: location_id.map(parse_uuid).transpose()?,
        name,
        description,
        aliases,
        keywords,
        category,
        barcode,
        material,
        color,
        condition_notes,
        quantity: quantity as i32,
        ai_raw: ai_raw_val,
        created_at: parse_datetime(created_at)?,
        updated_at: parse_datetime(updated_at)?,
    })
}

#[async_trait]
impl ItemRepository for SqliteItemRepository {
    async fn create(&self, group_id: Uuid, input: CreateItem) -> Result<Item> {
        let id = Uuid::new_v4();
        let id_str = id.to_string();
        let group_str = group_id.to_string();
        let (container_id_str, location_id_str) = match &input.parent {
            ParentRef::Container(cid) => (Some(cid.to_string()), None),
            ParentRef::Location(lid) => (None, Some(lid.to_string())),
        };
        let aliases_json = serde_json::to_string(&input.aliases.unwrap_or_default()).unwrap();
        let keywords_json = serde_json::to_string(&input.keywords.unwrap_or_default()).unwrap();
        let quantity = input.quantity.unwrap_or(1);

        sqlx::query!(
            r#"INSERT INTO items (id, group_id, container_id, location_id, name, description,
               aliases, keywords, category, barcode, material, color, condition_notes, quantity)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            id_str,
            group_str,
            container_id_str,
            location_id_str,
            input.name,
            input.description,
            aliases_json,
            keywords_json,
            input.category,
            input.barcode,
            input.material,
            input.color,
            input.condition_notes,
            quantity,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        self.get(id, group_id).await?.ok_or(DomainError::Internal(
            "failed to read back created item".into(),
        ))
    }

    async fn get(&self, id: Uuid, group_id: Uuid) -> Result<Option<Item>> {
        let id_str = id.to_string();
        let group_str = group_id.to_string();

        let row = sqlx::query!(
            r#"SELECT id, group_id, container_id, location_id, name, description,
               aliases, keywords, category, barcode, material, color, condition_notes,
               quantity, ai_raw, created_at, updated_at
               FROM items WHERE id = ? AND group_id = ?"#,
            id_str,
            group_str,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(row_to_item(
                &r.id,
                &r.group_id,
                r.container_id.as_deref(),
                r.location_id.as_deref(),
                r.name,
                r.description,
                &r.aliases,
                &r.keywords,
                r.category,
                r.barcode,
                r.material,
                r.color,
                r.condition_notes,
                r.quantity,
                r.ai_raw.as_deref(),
                &r.created_at,
                &r.updated_at,
            )?)),
            None => Ok(None),
        }
    }

    async fn update(&self, id: Uuid, group_id: Uuid, input: UpdateItem) -> Result<Item> {
        let existing = self.get(id, group_id).await?.ok_or(DomainError::NotFound {
            entity_type: "item".into(),
            id,
        })?;

        let name = input.name.unwrap_or(existing.name);
        let description = input.description.or(existing.description);
        let aliases = input.aliases.unwrap_or(existing.aliases);
        let keywords = input.keywords.unwrap_or(existing.keywords);
        let category = input.category.or(existing.category);
        let barcode = input.barcode.or(existing.barcode);
        let material = input.material.or(existing.material);
        let color = input.color.or(existing.color);
        let condition_notes = input.condition_notes.or(existing.condition_notes);
        let quantity = input.quantity.unwrap_or(existing.quantity);

        let aliases_json = serde_json::to_string(&aliases).unwrap();
        let keywords_json = serde_json::to_string(&keywords).unwrap();
        let id_str = id.to_string();
        let group_str = group_id.to_string();

        sqlx::query!(
            r#"UPDATE items SET name = ?, description = ?, aliases = ?, keywords = ?,
               category = ?, barcode = ?, material = ?, color = ?, condition_notes = ?,
               quantity = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%f', 'now')
               WHERE id = ? AND group_id = ?"#,
            name,
            description,
            aliases_json,
            keywords_json,
            category,
            barcode,
            material,
            color,
            condition_notes,
            quantity,
            id_str,
            group_str,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        self.get(id, group_id).await?.ok_or(DomainError::Internal(
            "failed to read back updated item".into(),
        ))
    }

    async fn delete(&self, id: Uuid, group_id: Uuid) -> Result<()> {
        self.get(id, group_id).await?.ok_or(DomainError::NotFound {
            entity_type: "item".into(),
            id,
        })?;

        let id_str = id.to_string();
        let group_str = group_id.to_string();

        sqlx::query!(
            "DELETE FROM items WHERE id = ? AND group_id = ?",
            id_str,
            group_str,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn move_to(&self, id: Uuid, group_id: Uuid, target: MoveTarget) -> Result<Item> {
        self.get(id, group_id).await?.ok_or(DomainError::NotFound {
            entity_type: "item".into(),
            id,
        })?;

        let id_str = id.to_string();
        let group_str = group_id.to_string();
        let (container_id_str, location_id_str) = match &target.target {
            ParentRef::Container(cid) => (Some(cid.to_string()), None),
            ParentRef::Location(lid) => (None, Some(lid.to_string())),
        };

        sqlx::query!(
            r#"UPDATE items SET container_id = ?, location_id = ?,
               updated_at = strftime('%Y-%m-%dT%H:%M:%f', 'now')
               WHERE id = ? AND group_id = ?"#,
            container_id_str,
            location_id_str,
            id_str,
            group_str,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        self.get(id, group_id).await?.ok_or(DomainError::Internal(
            "failed to read back moved item".into(),
        ))
    }

    async fn list_by_container(&self, container_id: Uuid, group_id: Uuid) -> Result<Vec<Item>> {
        let cont_str = container_id.to_string();
        let group_str = group_id.to_string();

        let rows = sqlx::query!(
            r#"SELECT id, group_id, container_id, location_id, name, description,
               aliases, keywords, category, barcode, material, color, condition_notes,
               quantity, ai_raw, created_at, updated_at
               FROM items WHERE container_id = ? AND group_id = ?
               ORDER BY name"#,
            cont_str,
            group_str,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                row_to_item(
                    &r.id,
                    &r.group_id,
                    r.container_id.as_deref(),
                    r.location_id.as_deref(),
                    r.name,
                    r.description,
                    &r.aliases,
                    &r.keywords,
                    r.category,
                    r.barcode,
                    r.material,
                    r.color,
                    r.condition_notes,
                    r.quantity,
                    r.ai_raw.as_deref(),
                    &r.created_at,
                    &r.updated_at,
                )
            })
            .collect()
    }

    async fn list_by_location(&self, location_id: Uuid, group_id: Uuid) -> Result<Vec<Item>> {
        let loc_str = location_id.to_string();
        let group_str = group_id.to_string();

        let rows = sqlx::query!(
            r#"SELECT id, group_id, container_id, location_id, name, description,
               aliases, keywords, category, barcode, material, color, condition_notes,
               quantity, ai_raw, created_at, updated_at
               FROM items WHERE location_id = ? AND group_id = ?
               ORDER BY name"#,
            loc_str,
            group_str,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                row_to_item(
                    &r.id,
                    &r.group_id,
                    r.container_id.as_deref(),
                    r.location_id.as_deref(),
                    r.name,
                    r.description,
                    &r.aliases,
                    &r.keywords,
                    r.category,
                    r.barcode,
                    r.material,
                    r.color,
                    r.condition_notes,
                    r.quantity,
                    r.ai_raw.as_deref(),
                    &r.created_at,
                    &r.updated_at,
                )
            })
            .collect()
    }

    async fn list_all(&self, group_id: Uuid) -> Result<Vec<Item>> {
        let group_str = group_id.to_string();

        let rows = sqlx::query!(
            r#"SELECT id, group_id, container_id, location_id, name, description,
               aliases, keywords, category, barcode, material, color, condition_notes,
               quantity, ai_raw, created_at, updated_at
               FROM items WHERE group_id = ?
               ORDER BY name"#,
            group_str,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                row_to_item(
                    &r.id,
                    &r.group_id,
                    r.container_id.as_deref(),
                    r.location_id.as_deref(),
                    r.name,
                    r.description,
                    &r.aliases,
                    &r.keywords,
                    r.category,
                    r.barcode,
                    r.material,
                    r.color,
                    r.condition_notes,
                    r.quantity,
                    r.ai_raw.as_deref(),
                    &r.created_at,
                    &r.updated_at,
                )
            })
            .collect()
    }

    async fn create_batch(&self, group_id: Uuid, items: Vec<CreateItem>) -> Result<Vec<Item>> {
        let mut results = Vec::with_capacity(items.len());
        // Use a transaction for atomicity
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        for input in items {
            let id = Uuid::new_v4();
            let id_str = id.to_string();
            let group_str = group_id.to_string();
            let (container_id_str, location_id_str) = match &input.parent {
                ParentRef::Container(cid) => (Some(cid.to_string()), None),
                ParentRef::Location(lid) => (None, Some(lid.to_string())),
            };
            let aliases_json = serde_json::to_string(&input.aliases.unwrap_or_default()).unwrap();
            let keywords_json = serde_json::to_string(&input.keywords.unwrap_or_default()).unwrap();
            let quantity = input.quantity.unwrap_or(1);

            sqlx::query!(
                r#"INSERT INTO items (id, group_id, container_id, location_id, name, description,
                   aliases, keywords, category, barcode, material, color, condition_notes, quantity)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
                id_str,
                group_str,
                container_id_str,
                location_id_str,
                input.name,
                input.description,
                aliases_json,
                keywords_json,
                input.category,
                input.barcode,
                input.material,
                input.color,
                input.condition_notes,
                quantity,
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

            results.push(id);
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        // Fetch all created items
        let mut created = Vec::with_capacity(results.len());
        for id in results {
            if let Some(item) = self.get(id, group_id).await? {
                created.push(item);
            }
        }
        Ok(created)
    }

    async fn count_by_container(&self, container_id: Uuid, group_id: Uuid) -> Result<i64> {
        let cont_str = container_id.to_string();
        let group_str = group_id.to_string();

        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM items WHERE container_id = ? AND group_id = ?",
            cont_str,
            group_str,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(count)
    }

    async fn count_by_location(&self, location_id: Uuid, group_id: Uuid) -> Result<i64> {
        let loc_str = location_id.to_string();
        let group_str = group_id.to_string();

        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM items WHERE location_id = ? AND group_id = ?",
            loc_str,
            group_str,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(count)
    }

    async fn list_all_unscoped(&self) -> Result<Vec<Item>> {
        let rows = sqlx::query!(
            r#"SELECT id, group_id, container_id, location_id, name, description,
               aliases, keywords, category, barcode, material, color, condition_notes,
               quantity, ai_raw, created_at, updated_at
               FROM items ORDER BY name"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|r| {
                row_to_item(
                    &r.id,
                    &r.group_id,
                    r.container_id.as_deref(),
                    r.location_id.as_deref(),
                    r.name,
                    r.description,
                    &r.aliases,
                    &r.keywords,
                    r.category,
                    r.barcode,
                    r.material,
                    r.color,
                    r.condition_notes,
                    r.quantity,
                    r.ai_raw.as_deref(),
                    &r.created_at,
                    &r.updated_at,
                )
            })
            .collect()
    }

    async fn insert_raw(&self, item: &Item) -> Result<()> {
        let id_str = item.id.to_string();
        let group_str = item.group_id.to_string();
        let container_str = item.container_id.map(|c| c.to_string());
        let location_str = item.location_id.map(|l| l.to_string());
        let aliases_json = serde_json::to_string(&item.aliases).unwrap();
        let keywords_json = serde_json::to_string(&item.keywords).unwrap();
        let ai_raw_json = item
            .ai_raw
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap());
        let created = item.created_at.format("%Y-%m-%dT%H:%M:%S%.3f").to_string();
        let updated = item.updated_at.format("%Y-%m-%dT%H:%M:%S%.3f").to_string();

        sqlx::query!(
            r#"INSERT INTO items (id, group_id, container_id, location_id, name, description,
               aliases, keywords, category, barcode, material, color, condition_notes,
               quantity, ai_raw, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            id_str,
            group_str,
            container_str,
            location_str,
            item.name,
            item.description,
            aliases_json,
            keywords_json,
            item.category,
            item.barcode,
            item.material,
            item.color,
            item.condition_notes,
            item.quantity,
            ai_raw_json,
            created,
            updated,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn delete_all(&self) -> Result<()> {
        sqlx::query!("DELETE FROM items")
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }
}
