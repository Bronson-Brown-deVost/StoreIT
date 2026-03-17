use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Discriminator for polymorphic references (photos, NFC tags, search index).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Location,
    Container,
    Item,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Location => "location",
            Self::Container => "container",
            Self::Item => "item",
        }
    }
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for EntityType {
    type Err = crate::errors::DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "location" => Ok(Self::Location),
            "container" => Ok(Self::Container),
            "item" => Ok(Self::Item),
            _ => Err(crate::errors::DomainError::InvalidEntityType(s.to_string())),
        }
    }
}

/// Encodes "either a Location or a Container" as a parent reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "id", rename_all = "snake_case")]
pub enum ParentRef {
    Location(Uuid),
    Container(Uuid),
}

// ---------------------------------------------------------------------------
// Location
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLocation {
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLocation {
    pub name: Option<String>,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

// ---------------------------------------------------------------------------
// Container
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Container {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContainer {
    pub parent: ParentRef,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateContainer {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
}

// ---------------------------------------------------------------------------
// Item
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
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
    pub ai_raw: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateItem {
    pub parent: ParentRef,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateItem {
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

/// Target for move operations on containers and items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveTarget {
    pub target: ParentRef,
}

// ---------------------------------------------------------------------------
// Photo
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Photo {
    pub id: Uuid,
    pub entity_type: EntityType,
    pub entity_id: Uuid,
    pub storage_key: String,
    pub thumbnail_key: Option<String>,
    pub large_key: Option<String>,
    pub mime_type: String,
    pub is_primary: bool,
    pub rotation: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePhoto {
    pub entity_type: EntityType,
    pub entity_id: Uuid,
    pub mime_type: String,
}

// ---------------------------------------------------------------------------
// NFC Tag
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NfcTag {
    pub id: Uuid,
    pub group_id: Uuid,
    pub tag_uri: String,
    pub entity_type: Option<EntityType>,
    pub entity_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub assigned_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub entity_type: EntityType,
    pub entity_id: Uuid,
    pub rank: f64,
}

// ---------------------------------------------------------------------------
// Hierarchy / Tree types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationTreeNode {
    pub location: Location,
    pub children: Vec<LocationTreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AncestryNode {
    pub entity_type: EntityType,
    pub id: Uuid,
    pub name: String,
}

// ---------------------------------------------------------------------------
// Auth: User, Group, Session
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupRole {
    Owner,
    Member,
}

impl GroupRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Member => "member",
        }
    }
}

impl std::fmt::Display for GroupRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for GroupRole {
    type Err = crate::errors::DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "owner" => Ok(Self::Owner),
            "member" => Ok(Self::Member),
            _ => Err(crate::errors::DomainError::Validation(format!(
                "invalid group role: {s}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub external_id: String,
    pub email: String,
    pub display_name: String,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub external_id: String,
    pub email: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUser {
    pub email: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLocalUser {
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
    pub is_admin: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserGroup {
    pub user_id: Uuid,
    pub group_id: Uuid,
    pub role: GroupRole,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: Uuid,
    pub active_group_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// AI Identification (trait defined now, Claude impl in M3)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentificationResult {
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub aliases: Vec<String>,
    pub keywords: Vec<String>,
    pub color: Option<String>,
    pub material: Option<String>,
    pub condition_notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_type_as_str() {
        assert_eq!(EntityType::Location.as_str(), "location");
        assert_eq!(EntityType::Container.as_str(), "container");
        assert_eq!(EntityType::Item.as_str(), "item");
    }

    #[test]
    fn entity_type_display() {
        assert_eq!(format!("{}", EntityType::Location), "location");
        assert_eq!(format!("{}", EntityType::Container), "container");
        assert_eq!(format!("{}", EntityType::Item), "item");
    }

    #[test]
    fn entity_type_from_str_valid() {
        assert_eq!(
            "location".parse::<EntityType>().unwrap(),
            EntityType::Location
        );
        assert_eq!(
            "container".parse::<EntityType>().unwrap(),
            EntityType::Container
        );
        assert_eq!("item".parse::<EntityType>().unwrap(), EntityType::Item);
    }

    #[test]
    fn entity_type_from_str_invalid() {
        let err = "bogus".parse::<EntityType>().unwrap_err();
        assert!(err.to_string().contains("bogus"));
    }

    #[test]
    fn entity_type_roundtrip_serde() {
        let val = EntityType::Container;
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "\"container\"");
        let back: EntityType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, val);
    }

    #[test]
    fn parent_ref_serde_location() {
        let id = Uuid::new_v4();
        let pr = ParentRef::Location(id);
        let json = serde_json::to_string(&pr).unwrap();
        assert!(json.contains("\"type\":\"location\""));
        let back: ParentRef = serde_json::from_str(&json).unwrap();
        assert_eq!(back, pr);
    }

    #[test]
    fn parent_ref_serde_container() {
        let id = Uuid::new_v4();
        let pr = ParentRef::Container(id);
        let json = serde_json::to_string(&pr).unwrap();
        assert!(json.contains("\"type\":\"container\""));
        let back: ParentRef = serde_json::from_str(&json).unwrap();
        assert_eq!(back, pr);
    }

    #[test]
    fn move_target_serde() {
        let id = Uuid::new_v4();
        let mt = MoveTarget {
            target: ParentRef::Location(id),
        };
        let json = serde_json::to_string(&mt).unwrap();
        let back: MoveTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(back.target, mt.target);
    }

    #[test]
    fn group_role_as_str() {
        assert_eq!(GroupRole::Owner.as_str(), "owner");
        assert_eq!(GroupRole::Member.as_str(), "member");
    }

    #[test]
    fn group_role_display() {
        assert_eq!(format!("{}", GroupRole::Owner), "owner");
        assert_eq!(format!("{}", GroupRole::Member), "member");
    }

    #[test]
    fn group_role_from_str_valid() {
        assert_eq!("owner".parse::<GroupRole>().unwrap(), GroupRole::Owner);
        assert_eq!("member".parse::<GroupRole>().unwrap(), GroupRole::Member);
    }

    #[test]
    fn group_role_from_str_invalid() {
        let err = "admin".parse::<GroupRole>().unwrap_err();
        assert!(err.to_string().contains("invalid group role"));
    }

    #[test]
    fn group_role_roundtrip_serde() {
        let val = GroupRole::Owner;
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "\"owner\"");
        let back: GroupRole = serde_json::from_str(&json).unwrap();
        assert_eq!(back, val);
    }

    #[test]
    fn user_serde_roundtrip() {
        let user = User {
            id: Uuid::new_v4(),
            external_id: "ext-123".into(),
            email: "test@example.com".into(),
            display_name: "Test User".into(),
            is_admin: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&user).unwrap();
        let back: User = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, user.id);
        assert_eq!(back.external_id, "ext-123");
    }

    #[test]
    fn group_serde_roundtrip() {
        let group = Group {
            id: Uuid::new_v4(),
            name: "family".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&group).unwrap();
        let back: Group = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "family");
    }

    #[test]
    fn session_serde_roundtrip() {
        let session = Session {
            id: "sess-abc".into(),
            user_id: Uuid::new_v4(),
            active_group_id: Uuid::new_v4(),
            expires_at: Utc::now(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&session).unwrap();
        let back: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "sess-abc");
    }

    // -- EntityType: additional coverage --

    #[test]
    fn entity_type_from_str_all_variants() {
        assert_eq!(
            "location".parse::<EntityType>().unwrap(),
            EntityType::Location
        );
        assert_eq!(
            "container".parse::<EntityType>().unwrap(),
            EntityType::Container
        );
        assert_eq!("item".parse::<EntityType>().unwrap(), EntityType::Item);
    }

    #[test]
    fn entity_type_from_str_case_sensitive() {
        assert!("Location".parse::<EntityType>().is_err());
        assert!("CONTAINER".parse::<EntityType>().is_err());
        assert!("Item".parse::<EntityType>().is_err());
    }

    #[test]
    fn entity_type_display_roundtrips_through_from_str() {
        for et in [
            EntityType::Location,
            EntityType::Container,
            EntityType::Item,
        ] {
            let s = et.to_string();
            let parsed: EntityType = s.parse().unwrap();
            assert_eq!(parsed, et);
        }
    }

    // -- GroupRole: additional coverage --

    #[test]
    fn group_role_from_str_case_sensitive() {
        assert!("Owner".parse::<GroupRole>().is_err());
        assert!("MEMBER".parse::<GroupRole>().is_err());
    }

    #[test]
    fn group_role_display_roundtrips_through_from_str() {
        for role in [GroupRole::Owner, GroupRole::Member] {
            let s = role.to_string();
            let parsed: GroupRole = s.parse().unwrap();
            assert_eq!(parsed, role);
        }
    }

    #[test]
    fn group_role_serde_member() {
        let val = GroupRole::Member;
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, "\"member\"");
        let back: GroupRole = serde_json::from_str(&json).unwrap();
        assert_eq!(back, val);
    }

    // -- Photo serde --

    #[test]
    fn photo_serde_roundtrip() {
        let photo = Photo {
            id: Uuid::new_v4(),
            entity_type: EntityType::Item,
            entity_id: Uuid::new_v4(),
            storage_key: "photos/abc.jpg".into(),
            thumbnail_key: Some("photos/abc_thumb.webp".into()),
            large_key: None,
            mime_type: "image/jpeg".into(),
            is_primary: true,
            rotation: 0,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&photo).unwrap();
        let back: Photo = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, photo.id);
        assert_eq!(back.entity_type, EntityType::Item);
        assert!(back.is_primary);
    }

    // -- NfcTag serde --

    #[test]
    fn nfc_tag_serde_roundtrip() {
        let tag = NfcTag {
            id: Uuid::new_v4(),
            group_id: Uuid::new_v4(),
            tag_uri: "urn:nfc:sn:AABB".into(),
            entity_type: Some(EntityType::Container),
            entity_id: Some(Uuid::new_v4()),
            created_at: Utc::now(),
            assigned_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&tag).unwrap();
        let back: NfcTag = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tag_uri, "urn:nfc:sn:AABB");
        assert_eq!(back.entity_type, Some(EntityType::Container));
    }

    #[test]
    fn nfc_tag_serde_unassigned() {
        let tag = NfcTag {
            id: Uuid::new_v4(),
            group_id: Uuid::new_v4(),
            tag_uri: "urn:nfc:sn:CCDD".into(),
            entity_type: None,
            entity_id: None,
            created_at: Utc::now(),
            assigned_at: None,
        };
        let json = serde_json::to_string(&tag).unwrap();
        let back: NfcTag = serde_json::from_str(&json).unwrap();
        assert_eq!(back.entity_type, None);
        assert_eq!(back.entity_id, None);
        assert_eq!(back.assigned_at, None);
    }

    // -- SearchResult serde --

    #[test]
    fn search_result_serde_roundtrip() {
        let sr = SearchResult {
            entity_type: EntityType::Location,
            entity_id: Uuid::new_v4(),
            rank: 0.95,
        };
        let json = serde_json::to_string(&sr).unwrap();
        let back: SearchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.entity_type, EntityType::Location);
        assert!((back.rank - 0.95).abs() < f64::EPSILON);
    }

    // -- IdentificationResult serde --

    #[test]
    fn identification_result_serde_roundtrip() {
        let ir = IdentificationResult {
            name: "USB Cable".into(),
            category: Some("Electronics".into()),
            description: Some("A USB-C cable".into()),
            aliases: vec!["USB-C cord".into()],
            keywords: vec!["cable".into(), "usb".into()],
            color: Some("black".into()),
            material: Some("plastic".into()),
            condition_notes: Some("good".into()),
        };
        let json = serde_json::to_string(&ir).unwrap();
        let back: IdentificationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "USB Cable");
        assert_eq!(back.aliases.len(), 1);
        assert_eq!(back.keywords.len(), 2);
    }

    // -- LocationTreeNode serde --

    #[test]
    fn location_tree_node_serde_roundtrip() {
        let node = LocationTreeNode {
            location: Location {
                id: Uuid::new_v4(),
                group_id: Uuid::new_v4(),
                parent_id: None,
                name: "Root".into(),
                description: None,
                latitude: None,
                longitude: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            children: vec![LocationTreeNode {
                location: Location {
                    id: Uuid::new_v4(),
                    group_id: Uuid::new_v4(),
                    parent_id: None,
                    name: "Child".into(),
                    description: None,
                    latitude: None,
                    longitude: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                children: vec![],
            }],
        };
        let json = serde_json::to_string(&node).unwrap();
        let back: LocationTreeNode = serde_json::from_str(&json).unwrap();
        assert_eq!(back.location.name, "Root");
        assert_eq!(back.children.len(), 1);
        assert_eq!(back.children[0].location.name, "Child");
    }

    // -- AncestryNode serde --

    #[test]
    fn ancestry_node_serde_roundtrip() {
        let node = AncestryNode {
            entity_type: EntityType::Container,
            id: Uuid::new_v4(),
            name: "Box A".into(),
        };
        let json = serde_json::to_string(&node).unwrap();
        let back: AncestryNode = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Box A");
        assert_eq!(back.entity_type, EntityType::Container);
    }

    // -- UserGroup serde --

    #[test]
    fn user_group_serde_roundtrip() {
        let ug = UserGroup {
            user_id: Uuid::new_v4(),
            group_id: Uuid::new_v4(),
            role: GroupRole::Owner,
        };
        let json = serde_json::to_string(&ug).unwrap();
        let back: UserGroup = serde_json::from_str(&json).unwrap();
        assert_eq!(back.role, GroupRole::Owner);
    }

    // -- Item serde --

    #[test]
    fn item_serde_roundtrip() {
        let item = Item {
            id: Uuid::new_v4(),
            group_id: Uuid::new_v4(),
            container_id: Some(Uuid::new_v4()),
            location_id: None,
            name: "Screwdriver".into(),
            description: Some("Phillips head".into()),
            aliases: vec!["driver".into()],
            keywords: vec!["tool".into()],
            category: Some("Tools".into()),
            barcode: Some("123456".into()),
            material: Some("steel".into()),
            color: Some("red".into()),
            condition_notes: Some("new".into()),
            quantity: 3,
            ai_raw: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&item).unwrap();
        let back: Item = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Screwdriver");
        assert_eq!(back.quantity, 3);
        assert_eq!(back.aliases, vec!["driver"]);
    }

    // -- Location serde --

    #[test]
    fn location_serde_roundtrip() {
        let loc = Location {
            id: Uuid::new_v4(),
            group_id: Uuid::new_v4(),
            parent_id: Some(Uuid::new_v4()),
            name: "Garage".into(),
            description: Some("Two car garage".into()),
            latitude: Some(47.6),
            longitude: Some(-122.3),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&loc).unwrap();
        let back: Location = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Garage");
        assert!(back.parent_id.is_some());
        assert!((back.latitude.unwrap() - 47.6).abs() < f64::EPSILON);
    }

    // -- Container serde --

    #[test]
    fn container_serde_roundtrip() {
        let c = Container {
            id: Uuid::new_v4(),
            group_id: Uuid::new_v4(),
            parent_location_id: Some(Uuid::new_v4()),
            parent_container_id: None,
            name: "Toolbox".into(),
            description: Some("Red toolbox".into()),
            color: Some("red".into()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: Container = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "Toolbox");
        assert_eq!(back.color, Some("red".into()));
    }
}
