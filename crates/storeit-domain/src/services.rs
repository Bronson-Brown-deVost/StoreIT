use uuid::Uuid;

use crate::entities::*;
use crate::errors::{DomainError, Result};
use crate::repositories::ContainerRepository;

/// Validates that moving a container does not create a circular reference.
pub async fn validate_container_move(
    container_repo: &dyn ContainerRepository,
    container_id: Uuid,
    target: &MoveTarget,
    group_id: Uuid,
) -> Result<()> {
    if let ParentRef::Container(target_container_id) = target.target {
        if target_container_id == container_id {
            return Err(DomainError::CircularReference {
                entity_type: "container".to_string(),
                id: container_id,
            });
        }
        if container_repo
            .is_ancestor_of(container_id, target_container_id, group_id)
            .await?
        {
            return Err(DomainError::CircularReference {
                entity_type: "container".to_string(),
                id: container_id,
            });
        }
    }
    Ok(())
}

/// Build searchable text blob from an item's fields.
pub fn build_item_search_text(item: &Item) -> String {
    let mut parts = vec![item.name.clone()];
    if let Some(ref desc) = item.description {
        parts.push(desc.clone());
    }
    if !item.aliases.is_empty() {
        parts.push(item.aliases.join(" "));
    }
    if !item.keywords.is_empty() {
        parts.push(item.keywords.join(" "));
    }
    if let Some(ref cat) = item.category {
        parts.push(cat.clone());
    }
    parts.join(" ")
}

/// Build searchable text blob from a location.
pub fn build_location_search_text(loc: &Location) -> String {
    let mut parts = vec![loc.name.clone()];
    if let Some(ref desc) = loc.description {
        parts.push(desc.clone());
    }
    parts.join(" ")
}

/// Build searchable text blob from a container.
pub fn build_container_search_text(c: &Container) -> String {
    let mut parts = vec![c.name.clone()];
    if let Some(ref desc) = c.description {
        parts.push(desc.clone());
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{Container, Item, Location, MoveTarget, ParentRef};
    use crate::errors::DomainError;
    use crate::repositories::ContainerRepository;
    use async_trait::async_trait;
    use chrono::Utc;
    use uuid::Uuid;

    // -- Mock ContainerRepository for testing validate_container_move --

    struct MockContainerRepo {
        is_ancestor_result: bool,
    }

    #[async_trait]
    impl ContainerRepository for MockContainerRepo {
        async fn create(&self, _: Uuid, _: CreateContainer) -> crate::errors::Result<Container> {
            unimplemented!()
        }
        async fn get(&self, _: Uuid, _: Uuid) -> crate::errors::Result<Option<Container>> {
            unimplemented!()
        }
        async fn update(
            &self,
            _: Uuid,
            _: Uuid,
            _: UpdateContainer,
        ) -> crate::errors::Result<Container> {
            unimplemented!()
        }
        async fn delete(&self, _: Uuid, _: Uuid) -> crate::errors::Result<()> {
            unimplemented!()
        }
        async fn move_to(
            &self,
            _: Uuid,
            _: Uuid,
            _: MoveTarget,
        ) -> crate::errors::Result<Container> {
            unimplemented!()
        }
        async fn list_by_location(
            &self,
            _: Uuid,
            _: Uuid,
        ) -> crate::errors::Result<Vec<Container>> {
            unimplemented!()
        }
        async fn list_by_container(
            &self,
            _: Uuid,
            _: Uuid,
        ) -> crate::errors::Result<Vec<Container>> {
            unimplemented!()
        }
        async fn list_all(&self, _: Uuid) -> crate::errors::Result<Vec<Container>> {
            unimplemented!()
        }
        async fn get_ancestry(&self, _: Uuid, _: Uuid) -> crate::errors::Result<Vec<AncestryNode>> {
            unimplemented!()
        }
        async fn has_children(&self, _: Uuid, _: Uuid) -> crate::errors::Result<bool> {
            unimplemented!()
        }
        async fn is_ancestor_of(
            &self,
            _ancestor_id: Uuid,
            _descendant_id: Uuid,
            _group_id: Uuid,
        ) -> crate::errors::Result<bool> {
            Ok(self.is_ancestor_result)
        }
        async fn list_all_unscoped(&self) -> crate::errors::Result<Vec<Container>> {
            unimplemented!()
        }
        async fn insert_raw(&self, _: &Container) -> crate::errors::Result<()> {
            unimplemented!()
        }
        async fn delete_all(&self) -> crate::errors::Result<()> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn validate_move_to_location_always_ok() {
        let repo = MockContainerRepo {
            is_ancestor_result: false,
        };
        let container_id = Uuid::new_v4();
        let target = MoveTarget {
            target: ParentRef::Location(Uuid::new_v4()),
        };
        let result = validate_container_move(&repo, container_id, &target, Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_move_to_self_rejected() {
        let repo = MockContainerRepo {
            is_ancestor_result: false,
        };
        let id = Uuid::new_v4();
        let target = MoveTarget {
            target: ParentRef::Container(id),
        };
        let result = validate_container_move(&repo, id, &target, Uuid::new_v4()).await;
        assert!(matches!(result, Err(DomainError::CircularReference { .. })));
    }

    #[tokio::test]
    async fn validate_move_to_descendant_rejected() {
        let repo = MockContainerRepo {
            is_ancestor_result: true,
        };
        let container_id = Uuid::new_v4();
        let target = MoveTarget {
            target: ParentRef::Container(Uuid::new_v4()),
        };
        let result = validate_container_move(&repo, container_id, &target, Uuid::new_v4()).await;
        assert!(matches!(result, Err(DomainError::CircularReference { .. })));
    }

    #[tokio::test]
    async fn validate_move_to_non_descendant_ok() {
        let repo = MockContainerRepo {
            is_ancestor_result: false,
        };
        let container_id = Uuid::new_v4();
        let target = MoveTarget {
            target: ParentRef::Container(Uuid::new_v4()),
        };
        let result = validate_container_move(&repo, container_id, &target, Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    fn make_item(name: &str) -> Item {
        Item {
            id: Uuid::new_v4(),
            group_id: Uuid::new_v4(),
            container_id: Some(Uuid::new_v4()),
            location_id: None,
            name: name.to_string(),
            description: None,
            aliases: vec![],
            keywords: vec![],
            category: None,
            barcode: None,
            material: None,
            color: None,
            condition_notes: None,
            quantity: 1,
            ai_raw: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn search_text_item_name_only() {
        let item = make_item("Cable Tie");
        assert_eq!(build_item_search_text(&item), "Cable Tie");
    }

    #[test]
    fn search_text_item_all_fields() {
        let mut item = make_item("Cable Tie");
        item.description = Some("Reusable nylon".into());
        item.aliases = vec!["zip tie".into(), "Kabelbinder".into()];
        item.keywords = vec!["fastener".into(), "cable".into()];
        item.category = Some("Hardware > Fasteners".into());
        let text = build_item_search_text(&item);
        assert!(text.contains("Cable Tie"));
        assert!(text.contains("Reusable nylon"));
        assert!(text.contains("zip tie"));
        assert!(text.contains("Kabelbinder"));
        assert!(text.contains("fastener"));
        assert!(text.contains("Hardware > Fasteners"));
    }

    #[test]
    fn search_text_location_name_only() {
        let loc = Location {
            id: Uuid::new_v4(),
            group_id: Uuid::new_v4(),
            parent_id: None,
            name: "Büro".into(),
            description: None,
            latitude: None,
            longitude: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(build_location_search_text(&loc), "Büro");
    }

    #[test]
    fn search_text_location_with_description() {
        let loc = Location {
            id: Uuid::new_v4(),
            group_id: Uuid::new_v4(),
            parent_id: None,
            name: "Büro".into(),
            description: Some("Home office room".into()),
            latitude: None,
            longitude: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let text = build_location_search_text(&loc);
        assert!(text.contains("Büro"));
        assert!(text.contains("Home office room"));
    }

    #[test]
    fn search_text_container_name_only() {
        let c = Container {
            id: Uuid::new_v4(),
            group_id: Uuid::new_v4(),
            parent_location_id: Some(Uuid::new_v4()),
            parent_container_id: None,
            name: "Box 7".into(),
            description: None,
            color: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(build_container_search_text(&c), "Box 7");
    }

    #[test]
    fn search_text_container_with_description() {
        let c = Container {
            id: Uuid::new_v4(),
            group_id: Uuid::new_v4(),
            parent_location_id: Some(Uuid::new_v4()),
            parent_container_id: None,
            name: "Box 7".into(),
            description: Some("Electronics parts".into()),
            color: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let text = build_container_search_text(&c);
        assert!(text.contains("Box 7"));
        assert!(text.contains("Electronics parts"));
    }
}
