use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use storeit_domain::entities;

// -- Error --

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(code: String, message: String) -> Self {
        Self {
            error: ErrorDetail { code, message },
        }
    }
}

// -- Location --

#[derive(Debug, Serialize, ToSchema)]
pub struct LocationResponse {
    pub id: Uuid,
    pub group_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<entities::Location> for LocationResponse {
    fn from(l: entities::Location) -> Self {
        Self {
            id: l.id,
            group_id: l.group_id,
            parent_id: l.parent_id,
            name: l.name,
            description: l.description,
            latitude: l.latitude,
            longitude: l.longitude,
            created_at: l.created_at,
            updated_at: l.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLocationRequest {
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateLocationRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LocationTreeNodeResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    #[schema(no_recursion)]
    pub children: Vec<LocationTreeNodeResponse>,
}

impl From<entities::LocationTreeNode> for LocationTreeNodeResponse {
    fn from(n: entities::LocationTreeNode) -> Self {
        Self {
            id: n.location.id,
            name: n.location.name,
            description: n.location.description,
            latitude: n.location.latitude,
            longitude: n.location.longitude,
            children: n.children.into_iter().map(Into::into).collect(),
        }
    }
}

// -- Container --

#[derive(Debug, Serialize, ToSchema)]
pub struct ContainerResponse {
    pub id: Uuid,
    pub group_id: Uuid,
    pub parent_location_id: Option<Uuid>,
    pub parent_container_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<entities::Container> for ContainerResponse {
    fn from(c: entities::Container) -> Self {
        Self {
            id: c.id,
            group_id: c.group_id,
            parent_location_id: c.parent_location_id,
            parent_container_id: c.parent_container_id,
            name: c.name,
            description: c.description,
            color: c.color,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateContainerRequest {
    pub parent_type: String,
    pub parent_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

impl CreateContainerRequest {
    pub fn to_parent_ref(
        &self,
    ) -> Result<entities::ParentRef, storeit_domain::errors::DomainError> {
        match self.parent_type.as_str() {
            "location" => Ok(entities::ParentRef::Location(self.parent_id)),
            "container" => Ok(entities::ParentRef::Container(self.parent_id)),
            other => Err(storeit_domain::errors::DomainError::Validation(format!(
                "invalid parent_type: {other}, must be 'location' or 'container'"
            ))),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateContainerRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MoveRequest {
    pub target_type: String,
    pub target_id: Uuid,
}

impl MoveRequest {
    pub fn to_parent_ref(
        &self,
    ) -> Result<entities::ParentRef, storeit_domain::errors::DomainError> {
        match self.target_type.as_str() {
            "location" => Ok(entities::ParentRef::Location(self.target_id)),
            "container" => Ok(entities::ParentRef::Container(self.target_id)),
            other => Err(storeit_domain::errors::DomainError::Validation(format!(
                "invalid target_type: {other}, must be 'location' or 'container'"
            ))),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AncestryNodeResponse {
    pub entity_type: String,
    pub id: Uuid,
    pub name: String,
}

impl From<entities::AncestryNode> for AncestryNodeResponse {
    fn from(n: entities::AncestryNode) -> Self {
        Self {
            entity_type: n.entity_type.as_str().to_string(),
            id: n.id,
            name: n.name,
        }
    }
}

// -- Item --

#[derive(Debug, Serialize, ToSchema)]
pub struct ItemResponse {
    pub id: Uuid,
    pub group_id: Uuid,
    pub container_id: Option<Uuid>,
    pub location_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub aliases: Vec<String>,
    pub keywords: Vec<String>,
    pub category: Option<String>,
    pub barcode: Option<String>,
    pub material: Option<String>,
    pub color: Option<String>,
    pub condition_notes: Option<String>,
    pub quantity: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<entities::Item> for ItemResponse {
    fn from(i: entities::Item) -> Self {
        Self {
            id: i.id,
            group_id: i.group_id,
            container_id: i.container_id,
            location_id: i.location_id,
            name: i.name,
            description: i.description,
            aliases: i.aliases,
            keywords: i.keywords,
            category: i.category,
            barcode: i.barcode,
            material: i.material,
            color: i.color,
            condition_notes: i.condition_notes,
            quantity: i.quantity,
            created_at: i.created_at,
            updated_at: i.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateItemRequest {
    pub parent_type: String,
    pub parent_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    pub category: Option<String>,
    pub barcode: Option<String>,
    pub material: Option<String>,
    pub color: Option<String>,
    pub condition_notes: Option<String>,
    pub quantity: Option<i32>,
}

impl CreateItemRequest {
    pub fn to_parent_ref(
        &self,
    ) -> Result<entities::ParentRef, storeit_domain::errors::DomainError> {
        match self.parent_type.as_str() {
            "location" => Ok(entities::ParentRef::Location(self.parent_id)),
            "container" => Ok(entities::ParentRef::Container(self.parent_id)),
            other => Err(storeit_domain::errors::DomainError::Validation(format!(
                "invalid parent_type: {other}"
            ))),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateItemRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    pub category: Option<String>,
    pub barcode: Option<String>,
    pub material: Option<String>,
    pub color: Option<String>,
    pub condition_notes: Option<String>,
    pub quantity: Option<i32>,
}

// -- Photo --

#[derive(Debug, Deserialize, ToSchema)]
pub struct RotatePhotoRequest {
    /// Rotation in degrees clockwise: 90, 180, or 270
    pub degrees: u16,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PhotoResponse {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub mime_type: String,
    pub is_primary: bool,
    pub rotation: i32,
    pub created_at: DateTime<Utc>,
}

impl From<entities::Photo> for PhotoResponse {
    fn from(p: entities::Photo) -> Self {
        Self {
            id: p.id,
            entity_type: p.entity_type.as_str().to_string(),
            entity_id: p.entity_id,
            mime_type: p.mime_type,
            is_primary: p.is_primary,
            rotation: p.rotation,
            created_at: p.created_at,
        }
    }
}

// -- Search --

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResultItem {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub score: f64,
}

impl From<entities::SearchResult> for SearchResultItem {
    fn from(r: entities::SearchResult) -> Self {
        Self {
            entity_type: r.entity_type.as_str().to_string(),
            entity_id: r.entity_id,
            score: r.rank,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResponse {
    pub results: Vec<SearchResultItem>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<u32>,
}

// -- Auth --

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub is_admin: bool,
}

impl From<entities::User> for UserResponse {
    fn from(u: entities::User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            display_name: u.display_name,
            is_admin: u.is_admin,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GroupResponse {
    pub id: Uuid,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MeResponse {
    pub user: UserResponse,
    pub groups: Vec<GroupResponse>,
    pub active_group_id: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SwitchGroupRequest {
    pub group_id: Uuid,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct AuthCallbackQuery {
    pub code: String,
    pub state: String,
}

// -- NFC Tags --

#[derive(Debug, Serialize, ToSchema)]
pub struct NfcTagResponse {
    pub id: Uuid,
    pub group_id: Uuid,
    pub tag_uri: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub assigned_at: Option<DateTime<Utc>>,
}

impl From<entities::NfcTag> for NfcTagResponse {
    fn from(t: entities::NfcTag) -> Self {
        Self {
            id: t.id,
            group_id: t.group_id,
            tag_uri: t.tag_uri,
            entity_type: t.entity_type.map(|et| et.as_str().to_string()),
            entity_id: t.entity_id,
            created_at: t.created_at,
            assigned_at: t.assigned_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateNfcTagRequest {
    pub tag_uri: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignNfcTagRequest {
    pub entity_type: String,
    pub entity_id: Uuid,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NfcResolveResponse {
    pub tag_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub entity_name: String,
    pub location_path: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NfcUidResolveResponse {
    /// "assigned", "unassigned", or "unknown"
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_path: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterAndAssignNfcRequest {
    pub tag_uri: String,
    pub entity_type: String,
    pub entity_id: Uuid,
}

// -- AI Identification --

#[derive(Debug, Serialize, ToSchema)]
pub struct IdentificationResponse {
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub aliases: Vec<String>,
    pub keywords: Vec<String>,
    pub color: Option<String>,
    pub material: Option<String>,
    pub condition_notes: Option<String>,
}

impl From<entities::IdentificationResult> for IdentificationResponse {
    fn from(r: entities::IdentificationResult) -> Self {
        Self {
            name: r.name,
            category: r.category,
            description: r.description,
            aliases: r.aliases,
            keywords: r.keywords,
            color: r.color,
            material: r.material,
            condition_notes: r.condition_notes,
        }
    }
}

// -- Auth Mode --

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthModeResponse {
    pub mode: String,
}

// -- Local Auth --

#[derive(Debug, Deserialize, ToSchema)]
pub struct LocalLoginRequest {
    pub username: String,
    pub password: String,
}

// -- Admin --

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminUserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

impl From<entities::User> for AdminUserResponse {
    fn from(u: entities::User) -> Self {
        let username = u
            .external_id
            .strip_prefix("local:")
            .unwrap_or(&u.external_id)
            .to_string();
        Self {
            id: u.id,
            username,
            email: u.email,
            display_name: u.display_name,
            is_admin: u.is_admin,
            created_at: u.created_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLocalUserRequest {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub password: String,
    pub is_admin: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateLocalUserRequest {
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub is_admin: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
    pub new_password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateGroupRequest {
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminGroupResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl From<entities::Group> for AdminGroupResponse {
    fn from(g: entities::Group) -> Self {
        Self {
            id: g.id,
            name: g.name,
            created_at: g.created_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
    pub role: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GroupMemberResponse {
    pub user: AdminUserResponse,
    pub role: String,
}

// -- Admin Settings --

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminSettingsResponse {
    pub image_storage_path: String,
    pub image_storage_path_readonly: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSettingsRequest {
    pub image_storage_path: String,
}

// -- Backup / Restore --

#[derive(Debug, Deserialize, ToSchema)]
pub struct BackupRequest {
    pub include_images: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BackupJobResponse {
    pub job_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JobStatusResponse {
    pub status: String,
    pub progress: u64,
    pub total: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RestoreOptions {
    pub mode: String,
    pub image_storage_path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    // -- ErrorResponse --

    #[test]
    fn error_response_new() {
        let resp = ErrorResponse::new("NOT_FOUND".into(), "not found".into());
        assert_eq!(resp.error.code, "NOT_FOUND");
        assert_eq!(resp.error.message, "not found");
    }

    // -- From<Location> for LocationResponse --

    #[test]
    fn location_response_from_entity() {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let gid = Uuid::new_v4();
        let pid = Uuid::new_v4();
        let loc = entities::Location {
            id,
            group_id: gid,
            parent_id: Some(pid),
            name: "Garage".into(),
            description: Some("The garage".into()),
            latitude: Some(47.6),
            longitude: Some(-122.3),
            created_at: now,
            updated_at: now,
        };
        let resp = LocationResponse::from(loc);
        assert_eq!(resp.id, id);
        assert_eq!(resp.group_id, gid);
        assert_eq!(resp.parent_id, Some(pid));
        assert_eq!(resp.name, "Garage");
        assert_eq!(resp.description, Some("The garage".into()));
        assert!((resp.latitude.unwrap() - 47.6).abs() < f64::EPSILON);
        assert!((resp.longitude.unwrap() + 122.3).abs() < f64::EPSILON);
    }

    // -- From<Container> for ContainerResponse --

    #[test]
    fn container_response_from_entity() {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let gid = Uuid::new_v4();
        let plid = Uuid::new_v4();
        let c = entities::Container {
            id,
            group_id: gid,
            parent_location_id: Some(plid),
            parent_container_id: None,
            name: "Box A".into(),
            description: Some("A box".into()),
            color: Some("blue".into()),
            created_at: now,
            updated_at: now,
        };
        let resp = ContainerResponse::from(c);
        assert_eq!(resp.id, id);
        assert_eq!(resp.group_id, gid);
        assert_eq!(resp.parent_location_id, Some(plid));
        assert_eq!(resp.parent_container_id, None);
        assert_eq!(resp.name, "Box A");
        assert_eq!(resp.color, Some("blue".into()));
    }

    // -- From<Item> for ItemResponse --

    #[test]
    fn item_response_from_entity() {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let gid = Uuid::new_v4();
        let cid = Uuid::new_v4();
        let item = entities::Item {
            id,
            group_id: gid,
            container_id: Some(cid),
            location_id: None,
            name: "Wrench".into(),
            description: Some("10mm".into()),
            aliases: vec!["spanner".into()],
            keywords: vec!["tool".into()],
            category: Some("Tools".into()),
            barcode: Some("789".into()),
            material: Some("steel".into()),
            color: Some("silver".into()),
            condition_notes: Some("good".into()),
            quantity: 2,
            ai_raw: None,
            created_at: now,
            updated_at: now,
        };
        let resp = ItemResponse::from(item);
        assert_eq!(resp.id, id);
        assert_eq!(resp.name, "Wrench");
        assert_eq!(resp.aliases, vec!["spanner"]);
        assert_eq!(resp.quantity, 2);
        assert_eq!(resp.barcode, Some("789".into()));
    }

    // -- From<Photo> for PhotoResponse --

    #[test]
    fn photo_response_from_entity() {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let eid = Uuid::new_v4();
        let photo = entities::Photo {
            id,
            entity_type: entities::EntityType::Item,
            entity_id: eid,
            storage_key: "photos/abc.jpg".into(),
            mime_type: "image/jpeg".into(),
            is_primary: true,
            rotation: 0,
            created_at: now,
        };
        let resp = PhotoResponse::from(photo);
        assert_eq!(resp.id, id);
        assert_eq!(resp.entity_type, "item");
        assert_eq!(resp.entity_id, eid);
        assert_eq!(resp.mime_type, "image/jpeg");
        assert!(resp.is_primary);
    }

    // -- From<SearchResult> for SearchResultItem --

    #[test]
    fn search_result_item_from_entity() {
        let eid = Uuid::new_v4();
        let sr = entities::SearchResult {
            entity_type: entities::EntityType::Container,
            entity_id: eid,
            rank: 0.85,
        };
        let resp = SearchResultItem::from(sr);
        assert_eq!(resp.entity_type, "container");
        assert_eq!(resp.entity_id, eid);
        assert!((resp.score - 0.85).abs() < f64::EPSILON);
    }

    // -- From<User> for UserResponse --

    #[test]
    fn user_response_from_entity() {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let user = entities::User {
            id,
            external_id: "ext-1".into(),
            email: "a@b.com".into(),
            display_name: "Alice".into(),
            is_admin: true,
            created_at: now,
            updated_at: now,
        };
        let resp = UserResponse::from(user);
        assert_eq!(resp.id, id);
        assert_eq!(resp.email, "a@b.com");
        assert_eq!(resp.display_name, "Alice");
        assert!(resp.is_admin);
    }

    // -- From<NfcTag> for NfcTagResponse --

    #[test]
    fn nfc_tag_response_from_entity_assigned() {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let gid = Uuid::new_v4();
        let eid = Uuid::new_v4();
        let tag = entities::NfcTag {
            id,
            group_id: gid,
            tag_uri: "urn:nfc:sn:AABB".into(),
            entity_type: Some(entities::EntityType::Item),
            entity_id: Some(eid),
            created_at: now,
            assigned_at: Some(now),
        };
        let resp = NfcTagResponse::from(tag);
        assert_eq!(resp.id, id);
        assert_eq!(resp.tag_uri, "urn:nfc:sn:AABB");
        assert_eq!(resp.entity_type, Some("item".into()));
        assert_eq!(resp.entity_id, Some(eid));
        assert!(resp.assigned_at.is_some());
    }

    #[test]
    fn nfc_tag_response_from_entity_unassigned() {
        let now = Utc::now();
        let tag = entities::NfcTag {
            id: Uuid::new_v4(),
            group_id: Uuid::new_v4(),
            tag_uri: "urn:nfc:sn:CCDD".into(),
            entity_type: None,
            entity_id: None,
            created_at: now,
            assigned_at: None,
        };
        let resp = NfcTagResponse::from(tag);
        assert_eq!(resp.entity_type, None);
        assert_eq!(resp.entity_id, None);
        assert_eq!(resp.assigned_at, None);
    }

    // -- From<IdentificationResult> for IdentificationResponse --

    #[test]
    fn identification_response_from_entity() {
        let ir = entities::IdentificationResult {
            name: "USB Cable".into(),
            category: Some("Electronics".into()),
            description: Some("A USB-C cable".into()),
            aliases: vec!["USB-C cord".into()],
            keywords: vec!["cable".into(), "usb".into()],
            color: Some("black".into()),
            material: Some("plastic".into()),
            condition_notes: Some("good".into()),
        };
        let resp = IdentificationResponse::from(ir);
        assert_eq!(resp.name, "USB Cable");
        assert_eq!(resp.category, Some("Electronics".into()));
        assert_eq!(resp.aliases, vec!["USB-C cord"]);
        assert_eq!(resp.keywords.len(), 2);
    }

    // -- From<LocationTreeNode> for LocationTreeNodeResponse --

    #[test]
    fn location_tree_node_response_from_entity() {
        let now = Utc::now();
        let node = entities::LocationTreeNode {
            location: entities::Location {
                id: Uuid::new_v4(),
                group_id: Uuid::new_v4(),
                parent_id: None,
                name: "Root".into(),
                description: Some("Root loc".into()),
                latitude: Some(1.0),
                longitude: Some(2.0),
                created_at: now,
                updated_at: now,
            },
            children: vec![entities::LocationTreeNode {
                location: entities::Location {
                    id: Uuid::new_v4(),
                    group_id: Uuid::new_v4(),
                    parent_id: None,
                    name: "Child".into(),
                    description: None,
                    latitude: None,
                    longitude: None,
                    created_at: now,
                    updated_at: now,
                },
                children: vec![],
            }],
        };
        let resp = LocationTreeNodeResponse::from(node);
        assert_eq!(resp.name, "Root");
        assert_eq!(resp.description, Some("Root loc".into()));
        assert_eq!(resp.children.len(), 1);
        assert_eq!(resp.children[0].name, "Child");
        assert!(resp.children[0].children.is_empty());
    }

    // -- From<AncestryNode> for AncestryNodeResponse --

    #[test]
    fn ancestry_node_response_from_entity() {
        let id = Uuid::new_v4();
        let node = entities::AncestryNode {
            entity_type: entities::EntityType::Location,
            id,
            name: "Basement".into(),
        };
        let resp = AncestryNodeResponse::from(node);
        assert_eq!(resp.entity_type, "location");
        assert_eq!(resp.id, id);
        assert_eq!(resp.name, "Basement");
    }

    // -- From<User> for AdminUserResponse --

    #[test]
    fn admin_user_response_from_entity_local() {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let user = entities::User {
            id,
            external_id: "local:admin".into(),
            email: "admin@test.com".into(),
            display_name: "Admin".into(),
            is_admin: true,
            created_at: now,
            updated_at: now,
        };
        let resp = AdminUserResponse::from(user);
        assert_eq!(resp.id, id);
        assert_eq!(resp.username, "admin");
        assert!(resp.is_admin);
    }

    #[test]
    fn admin_user_response_from_entity_oidc() {
        let now = Utc::now();
        let user = entities::User {
            id: Uuid::new_v4(),
            external_id: "oidc-sub-123".into(),
            email: "user@test.com".into(),
            display_name: "User".into(),
            is_admin: false,
            created_at: now,
            updated_at: now,
        };
        let resp = AdminUserResponse::from(user);
        // No "local:" prefix, so username == external_id
        assert_eq!(resp.username, "oidc-sub-123");
        assert!(!resp.is_admin);
    }

    // -- From<Group> for AdminGroupResponse --

    #[test]
    fn admin_group_response_from_entity() {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let group = entities::Group {
            id,
            name: "family".into(),
            created_at: now,
            updated_at: now,
        };
        let resp = AdminGroupResponse::from(group);
        assert_eq!(resp.id, id);
        assert_eq!(resp.name, "family");
    }

    // -- CreateContainerRequest::to_parent_ref --

    #[test]
    fn create_container_request_to_parent_ref_location() {
        let id = Uuid::new_v4();
        let req = CreateContainerRequest {
            parent_type: "location".into(),
            parent_id: id,
            name: "Box".into(),
            description: None,
            color: None,
        };
        let pr = req.to_parent_ref().unwrap();
        assert_eq!(pr, entities::ParentRef::Location(id));
    }

    #[test]
    fn create_container_request_to_parent_ref_container() {
        let id = Uuid::new_v4();
        let req = CreateContainerRequest {
            parent_type: "container".into(),
            parent_id: id,
            name: "Box".into(),
            description: None,
            color: None,
        };
        let pr = req.to_parent_ref().unwrap();
        assert_eq!(pr, entities::ParentRef::Container(id));
    }

    #[test]
    fn create_container_request_to_parent_ref_invalid() {
        let req = CreateContainerRequest {
            parent_type: "item".into(),
            parent_id: Uuid::new_v4(),
            name: "Box".into(),
            description: None,
            color: None,
        };
        let err = req.to_parent_ref().unwrap_err();
        assert!(err.to_string().contains("invalid parent_type"));
    }

    // -- CreateItemRequest::to_parent_ref --

    #[test]
    fn create_item_request_to_parent_ref_location() {
        let id = Uuid::new_v4();
        let req = CreateItemRequest {
            parent_type: "location".into(),
            parent_id: id,
            name: "Drill".into(),
            description: None,
            aliases: None,
            keywords: None,
            category: None,
            barcode: None,
            material: None,
            color: None,
            condition_notes: None,
            quantity: None,
        };
        let pr = req.to_parent_ref().unwrap();
        assert_eq!(pr, entities::ParentRef::Location(id));
    }

    #[test]
    fn create_item_request_to_parent_ref_container() {
        let id = Uuid::new_v4();
        let req = CreateItemRequest {
            parent_type: "container".into(),
            parent_id: id,
            name: "Drill".into(),
            description: None,
            aliases: None,
            keywords: None,
            category: None,
            barcode: None,
            material: None,
            color: None,
            condition_notes: None,
            quantity: None,
        };
        let pr = req.to_parent_ref().unwrap();
        assert_eq!(pr, entities::ParentRef::Container(id));
    }

    #[test]
    fn create_item_request_to_parent_ref_invalid() {
        let req = CreateItemRequest {
            parent_type: "bogus".into(),
            parent_id: Uuid::new_v4(),
            name: "Drill".into(),
            description: None,
            aliases: None,
            keywords: None,
            category: None,
            barcode: None,
            material: None,
            color: None,
            condition_notes: None,
            quantity: None,
        };
        let err = req.to_parent_ref().unwrap_err();
        assert!(err.to_string().contains("invalid parent_type"));
    }

    // -- MoveRequest::to_parent_ref --

    #[test]
    fn move_request_to_parent_ref_location() {
        let id = Uuid::new_v4();
        let req = MoveRequest {
            target_type: "location".into(),
            target_id: id,
        };
        let pr = req.to_parent_ref().unwrap();
        assert_eq!(pr, entities::ParentRef::Location(id));
    }

    #[test]
    fn move_request_to_parent_ref_container() {
        let id = Uuid::new_v4();
        let req = MoveRequest {
            target_type: "container".into(),
            target_id: id,
        };
        let pr = req.to_parent_ref().unwrap();
        assert_eq!(pr, entities::ParentRef::Container(id));
    }

    #[test]
    fn move_request_to_parent_ref_invalid() {
        let req = MoveRequest {
            target_type: "invalid".into(),
            target_id: Uuid::new_v4(),
        };
        let err = req.to_parent_ref().unwrap_err();
        assert!(err.to_string().contains("invalid target_type"));
    }
}
