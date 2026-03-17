use async_trait::async_trait;
use uuid::Uuid;

use crate::entities::*;
use crate::errors::Result;

#[async_trait]
pub trait LocationRepository: Send + Sync {
    async fn create(&self, group_id: Uuid, input: CreateLocation) -> Result<Location>;
    async fn get(&self, id: Uuid, group_id: Uuid) -> Result<Option<Location>>;
    async fn update(&self, id: Uuid, group_id: Uuid, input: UpdateLocation) -> Result<Location>;
    async fn delete(&self, id: Uuid, group_id: Uuid) -> Result<()>;
    async fn list_roots(&self, group_id: Uuid) -> Result<Vec<Location>>;
    async fn list_children(&self, parent_id: Uuid, group_id: Uuid) -> Result<Vec<Location>>;
    async fn get_tree(&self, group_id: Uuid) -> Result<Vec<LocationTreeNode>>;
    async fn has_children(&self, id: Uuid, group_id: Uuid) -> Result<bool>;
    async fn list_all_unscoped(&self) -> Result<Vec<Location>>;
    async fn insert_raw(&self, location: &Location) -> Result<()>;
    async fn delete_all(&self) -> Result<()>;
}

#[async_trait]
pub trait ContainerRepository: Send + Sync {
    async fn create(&self, group_id: Uuid, input: CreateContainer) -> Result<Container>;
    async fn get(&self, id: Uuid, group_id: Uuid) -> Result<Option<Container>>;
    async fn update(&self, id: Uuid, group_id: Uuid, input: UpdateContainer) -> Result<Container>;
    async fn delete(&self, id: Uuid, group_id: Uuid) -> Result<()>;
    async fn move_to(&self, id: Uuid, group_id: Uuid, target: MoveTarget) -> Result<Container>;
    async fn list_by_location(&self, location_id: Uuid, group_id: Uuid) -> Result<Vec<Container>>;
    async fn list_by_container(&self, container_id: Uuid, group_id: Uuid)
    -> Result<Vec<Container>>;
    async fn list_all(&self, group_id: Uuid) -> Result<Vec<Container>>;
    async fn get_ancestry(&self, id: Uuid, group_id: Uuid) -> Result<Vec<AncestryNode>>;
    async fn has_children(&self, id: Uuid, group_id: Uuid) -> Result<bool>;
    async fn is_ancestor_of(
        &self,
        ancestor_id: Uuid,
        descendant_id: Uuid,
        group_id: Uuid,
    ) -> Result<bool>;
    async fn list_all_unscoped(&self) -> Result<Vec<Container>>;
    async fn insert_raw(&self, container: &Container) -> Result<()>;
    async fn delete_all(&self) -> Result<()>;
}

#[async_trait]
pub trait ItemRepository: Send + Sync {
    async fn create(&self, group_id: Uuid, input: CreateItem) -> Result<Item>;
    async fn get(&self, id: Uuid, group_id: Uuid) -> Result<Option<Item>>;
    async fn update(&self, id: Uuid, group_id: Uuid, input: UpdateItem) -> Result<Item>;
    async fn delete(&self, id: Uuid, group_id: Uuid) -> Result<()>;
    async fn move_to(&self, id: Uuid, group_id: Uuid, target: MoveTarget) -> Result<Item>;
    async fn list_by_container(&self, container_id: Uuid, group_id: Uuid) -> Result<Vec<Item>>;
    async fn list_by_location(&self, location_id: Uuid, group_id: Uuid) -> Result<Vec<Item>>;
    async fn create_batch(&self, group_id: Uuid, items: Vec<CreateItem>) -> Result<Vec<Item>>;
    async fn list_all(&self, group_id: Uuid) -> Result<Vec<Item>>;
    async fn count_by_container(&self, container_id: Uuid, group_id: Uuid) -> Result<i64>;
    async fn count_by_location(&self, location_id: Uuid, group_id: Uuid) -> Result<i64>;
    async fn list_all_unscoped(&self) -> Result<Vec<Item>>;
    async fn insert_raw(&self, item: &Item) -> Result<()>;
    async fn delete_all(&self) -> Result<()>;
}

#[async_trait]
pub trait PhotoRepository: Send + Sync {
    async fn create(&self, input: CreatePhoto, storage_key: String) -> Result<Photo>;
    async fn get(&self, id: Uuid) -> Result<Option<Photo>>;
    async fn list_by_entity(&self, entity_type: EntityType, entity_id: Uuid) -> Result<Vec<Photo>>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn set_primary(&self, id: Uuid, entity_type: EntityType, entity_id: Uuid) -> Result<()>;
    async fn list_all(&self) -> Result<Vec<Photo>>;
    async fn count_by_storage_key(&self, storage_key: &str) -> Result<i64>;
    async fn update_storage_key(&self, id: Uuid, new_key: &str) -> Result<()>;
    async fn set_rotation(&self, id: Uuid, rotation: i32) -> Result<()>;
    async fn insert_raw(&self, photo: &Photo) -> Result<()>;
    async fn delete_all(&self) -> Result<()>;
}

#[async_trait]
pub trait NfcTagRepository: Send + Sync {
    async fn create(&self, group_id: Uuid, tag_uri: String) -> Result<NfcTag>;
    async fn get(&self, id: Uuid) -> Result<Option<NfcTag>>;
    async fn get_by_uri(&self, tag_uri: &str) -> Result<Option<NfcTag>>;
    async fn list_by_group(&self, group_id: Uuid) -> Result<Vec<NfcTag>>;
    async fn list_by_entity(&self, entity_type: EntityType, entity_id: Uuid)
    -> Result<Vec<NfcTag>>;
    async fn assign(&self, id: Uuid, entity_type: EntityType, entity_id: Uuid) -> Result<NfcTag>;
    async fn unassign(&self, id: Uuid) -> Result<()>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn list_all_unscoped(&self) -> Result<Vec<NfcTag>>;
    async fn insert_raw(&self, tag: &NfcTag) -> Result<()>;
    async fn delete_all(&self) -> Result<()>;
}

#[async_trait]
pub trait SearchRepository: Send + Sync {
    async fn index(
        &self,
        entity_type: EntityType,
        entity_id: Uuid,
        group_id: Uuid,
        text: &str,
    ) -> Result<()>;
    async fn remove(&self, entity_type: EntityType, entity_id: Uuid) -> Result<()>;
    async fn search(&self, group_id: Uuid, query: &str, limit: u32) -> Result<Vec<SearchResult>>;
    async fn rebuild_index(&self) -> Result<()>;
    async fn full_reindex(
        &self,
        locations: &[crate::entities::Location],
        containers: &[crate::entities::Container],
        items: &[crate::entities::Item],
    ) -> Result<()>;
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn upsert_by_external_id(&self, input: CreateUser) -> Result<User>;
    async fn get(&self, id: Uuid) -> Result<Option<User>>;
    async fn get_by_external_id(&self, external_id: &str) -> Result<Option<User>>;
    async fn create_local(&self, input: CreateLocalUser) -> Result<User>;
    async fn get_password_hash(&self, external_id: &str) -> Result<Option<String>>;
    async fn set_password_hash(&self, id: Uuid, hash: &str) -> Result<()>;
    async fn set_admin(&self, id: Uuid, is_admin: bool) -> Result<()>;
    async fn list_all(&self) -> Result<Vec<User>>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn count_admins(&self) -> Result<i64>;
    async fn insert_raw(&self, user: &User, password_hash: Option<&str>) -> Result<()>;
    async fn delete_all(&self) -> Result<()>;
}

#[async_trait]
pub trait GroupRepository: Send + Sync {
    async fn get_or_create_by_name(&self, name: &str) -> Result<Group>;
    async fn get(&self, id: Uuid) -> Result<Option<Group>>;
    async fn list_all(&self) -> Result<Vec<Group>>;
    async fn create(&self, name: &str) -> Result<Group>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn insert_raw(&self, group: &Group) -> Result<()>;
    async fn delete_all(&self) -> Result<()>;
}

#[async_trait]
pub trait UserGroupRepository: Send + Sync {
    async fn set_memberships(&self, user_id: Uuid, groups: Vec<(Uuid, GroupRole)>) -> Result<()>;
    async fn list_groups_for_user(&self, user_id: Uuid) -> Result<Vec<(Group, GroupRole)>>;
    async fn is_member(&self, user_id: Uuid, group_id: Uuid) -> Result<bool>;
    async fn add_member(&self, user_id: Uuid, group_id: Uuid, role: GroupRole) -> Result<()>;
    async fn remove_member(&self, user_id: Uuid, group_id: Uuid) -> Result<()>;
    async fn list_members_of_group(&self, group_id: Uuid) -> Result<Vec<(User, GroupRole)>>;
    async fn list_all(&self) -> Result<Vec<UserGroup>>;
    async fn insert_raw(&self, user_id: Uuid, group_id: Uuid, role: GroupRole) -> Result<()>;
    async fn delete_all(&self) -> Result<()>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create(&self, session: Session) -> Result<Session>;
    async fn get(&self, id: &str) -> Result<Option<Session>>;
    async fn update_active_group(&self, id: &str, group_id: Uuid) -> Result<Session>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn delete_expired(&self) -> Result<u64>;
    async fn delete_all(&self) -> Result<()>;
}

#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>>;
    async fn set(&self, key: &str, value: &str) -> Result<()>;
    async fn list_all(&self) -> Result<Vec<(String, String)>>;
    async fn delete_all(&self) -> Result<()>;
}
