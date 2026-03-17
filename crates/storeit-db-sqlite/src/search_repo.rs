use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use storeit_domain::entities::*;
use storeit_domain::errors::{DomainError, Result};
use storeit_domain::repositories::SearchRepository;
use storeit_domain::services::{
    build_container_search_text, build_item_search_text, build_location_search_text,
};
use uuid::Uuid;

pub struct SqliteSearchRepository {
    pool: SqlitePool,
}

impl SqliteSearchRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn parse_uuid(s: &str) -> Result<Uuid> {
    s.parse()
        .map_err(|e: uuid::Error| DomainError::Internal(e.to_string()))
}

#[async_trait]
impl SearchRepository for SqliteSearchRepository {
    async fn index(
        &self,
        entity_type: EntityType,
        entity_id: Uuid,
        group_id: Uuid,
        text: &str,
    ) -> Result<()> {
        let et_str = entity_type.as_str().to_string();
        let eid_str = entity_id.to_string();
        let gid_str = group_id.to_string();
        let text = text.to_string();

        // Remove existing entry first (upsert)
        sqlx::query("DELETE FROM search_index WHERE entity_type = ? AND entity_id = ?")
            .bind(&et_str)
            .bind(&eid_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        sqlx::query(
            "INSERT INTO search_index (entity_type, entity_id, group_id, searchable_text) VALUES (?, ?, ?, ?)",
        )
        .bind(&et_str)
        .bind(&eid_str)
        .bind(&gid_str)
        .bind(&text)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn remove(&self, entity_type: EntityType, entity_id: Uuid) -> Result<()> {
        let et_str = entity_type.as_str().to_string();
        let eid_str = entity_id.to_string();

        sqlx::query("DELETE FROM search_index WHERE entity_type = ? AND entity_id = ?")
            .bind(&et_str)
            .bind(&eid_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn search(&self, group_id: Uuid, query: &str, limit: u32) -> Result<Vec<SearchResult>> {
        let gid_str = group_id.to_string();

        let rows = sqlx::query(
            r#"SELECT entity_type, entity_id, rank
               FROM search_index
               WHERE search_index MATCH ?1
                 AND group_id = ?2
               ORDER BY rank
               LIMIT ?3"#,
        )
        .bind(query)
        .bind(&gid_str)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|row| {
                let et: String = row.get("entity_type");
                let eid: String = row.get("entity_id");
                let rank: f64 = row.get("rank");

                Ok(SearchResult {
                    entity_type: et.parse()?,
                    entity_id: parse_uuid(&eid)?,
                    rank,
                })
            })
            .collect()
    }

    async fn rebuild_index(&self) -> Result<()> {
        sqlx::query("DELETE FROM search_index")
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;
        Ok(())
    }

    async fn full_reindex(
        &self,
        locations: &[storeit_domain::entities::Location],
        containers: &[storeit_domain::entities::Container],
        items: &[storeit_domain::entities::Item],
    ) -> Result<()> {
        // Clear existing index
        self.rebuild_index().await?;

        // Re-index locations
        for l in locations {
            let text = build_location_search_text(l);
            self.index(EntityType::Location, l.id, l.group_id, &text)
                .await?;
        }

        // Re-index containers
        for c in containers {
            let text = build_container_search_text(c);
            self.index(EntityType::Container, c.id, c.group_id, &text)
                .await?;
        }

        // Re-index items
        for i in items {
            let text = build_item_search_text(i);
            self.index(EntityType::Item, i.id, i.group_id, &text)
                .await?;
        }

        Ok(())
    }
}
