use sqlx::SqlitePool;
use storeit_db_sqlite::*;
use storeit_domain::entities::*;
use storeit_domain::repositories::*;
use uuid::Uuid;

async fn test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect to in-memory sqlite");
    let db = SqliteDb::new(pool.clone());
    db.migrate().await.expect("run migrations");
    pool
}

fn group_id() -> Uuid {
    Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
}

// =========================================================================
// Location Repository Tests
// =========================================================================

#[tokio::test]
async fn location_create_and_get() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);
    let loc = repo
        .create(
            group_id(),
            CreateLocation {
                parent_id: None,
                name: "Living Room".into(),
                description: Some("Main floor".into()),
                latitude: None,
                longitude: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(loc.name, "Living Room");
    assert_eq!(loc.description.as_deref(), Some("Main floor"));
    assert!(loc.parent_id.is_none());

    let fetched = repo.get(loc.id, group_id()).await.unwrap().unwrap();
    assert_eq!(fetched.id, loc.id);
    assert_eq!(fetched.name, "Living Room");
}

#[tokio::test]
async fn location_create_with_coordinates() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);
    let loc = repo
        .create(
            group_id(),
            CreateLocation {
                parent_id: None,
                name: "Cabin".into(),
                description: Some("Mountain cabin".into()),
                latitude: Some(45.123456),
                longitude: Some(-122.654321),
            },
        )
        .await
        .unwrap();

    assert_eq!(loc.name, "Cabin");
    assert!((loc.latitude.unwrap() - 45.123456).abs() < 1e-6);
    assert!((loc.longitude.unwrap() - (-122.654321)).abs() < 1e-6);

    let fetched = repo.get(loc.id, group_id()).await.unwrap().unwrap();
    assert!((fetched.latitude.unwrap() - 45.123456).abs() < 1e-6);
    assert!((fetched.longitude.unwrap() - (-122.654321)).abs() < 1e-6);
}

#[tokio::test]
async fn location_create_without_coordinates() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);
    let loc = repo
        .create(
            group_id(),
            CreateLocation {
                parent_id: None,
                name: "No GPS".into(),
                description: None,
                latitude: None,
                longitude: None,
            },
        )
        .await
        .unwrap();

    assert!(loc.latitude.is_none());
    assert!(loc.longitude.is_none());
}

#[tokio::test]
async fn location_update_coordinates() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);
    let loc = repo
        .create(
            group_id(),
            CreateLocation {
                parent_id: None,
                name: "Home".into(),
                description: None,
                latitude: None,
                longitude: None,
            },
        )
        .await
        .unwrap();

    assert!(loc.latitude.is_none());

    let updated = repo
        .update(
            loc.id,
            group_id(),
            UpdateLocation {
                name: None,
                description: None,
                latitude: Some(34.0522),
                longitude: Some(-118.2437),
            },
        )
        .await
        .unwrap();

    assert!((updated.latitude.unwrap() - 34.0522).abs() < 1e-4);
    assert!((updated.longitude.unwrap() - (-118.2437)).abs() < 1e-4);

    // Verify via get
    let fetched = repo.get(loc.id, group_id()).await.unwrap().unwrap();
    assert!((fetched.latitude.unwrap() - 34.0522).abs() < 1e-4);
    assert!((fetched.longitude.unwrap() - (-118.2437)).abs() < 1e-4);
}

#[tokio::test]
async fn location_coordinates_in_tree() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);
    repo.create(
        group_id(),
        CreateLocation {
            parent_id: None,
            name: "GPS Root".into(),
            description: None,
            latitude: Some(51.5074),
            longitude: Some(-0.1278),
        },
    )
    .await
    .unwrap();

    let tree = repo.get_tree(group_id()).await.unwrap();
    assert_eq!(tree.len(), 1);
    assert!((tree[0].location.latitude.unwrap() - 51.5074).abs() < 1e-4);
    assert!((tree[0].location.longitude.unwrap() - (-0.1278)).abs() < 1e-4);
}

#[tokio::test]
async fn location_coordinates_in_list_roots() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);
    repo.create(
        group_id(),
        CreateLocation {
            parent_id: None,
            name: "GPS Root".into(),
            description: None,
            latitude: Some(48.8566),
            longitude: Some(2.3522),
        },
    )
    .await
    .unwrap();

    let roots = repo.list_roots(group_id()).await.unwrap();
    assert_eq!(roots.len(), 1);
    assert!((roots[0].latitude.unwrap() - 48.8566).abs() < 1e-4);
    assert!((roots[0].longitude.unwrap() - 2.3522).abs() < 1e-4);
}

#[tokio::test]
async fn location_coordinates_in_list_children() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);
    let parent = repo
        .create(
            group_id(),
            CreateLocation {
                parent_id: None,
                name: "Parent".into(),
                description: None,
                latitude: None,
                longitude: None,
            },
        )
        .await
        .unwrap();
    repo.create(
        group_id(),
        CreateLocation {
            parent_id: Some(parent.id),
            name: "Child with GPS".into(),
            description: None,
            latitude: Some(35.6762),
            longitude: Some(139.6503),
        },
    )
    .await
    .unwrap();

    let children = repo.list_children(parent.id, group_id()).await.unwrap();
    assert_eq!(children.len(), 1);
    assert!((children[0].latitude.unwrap() - 35.6762).abs() < 1e-4);
    assert!((children[0].longitude.unwrap() - 139.6503).abs() < 1e-4);
}

#[tokio::test]
async fn location_get_not_found() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);
    let result = repo.get(Uuid::new_v4(), group_id()).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn location_update() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);
    let loc = repo
        .create(
            group_id(),
            CreateLocation {
                parent_id: None,
                name: "Office".into(),
                description: None,
                latitude: None,
                longitude: None,
            },
        )
        .await
        .unwrap();

    let updated = repo
        .update(
            loc.id,
            group_id(),
            UpdateLocation {
                name: Some("Home Office".into()),
                description: Some("Upstairs".into()),
                latitude: None,
                longitude: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.name, "Home Office");
    assert_eq!(updated.description.as_deref(), Some("Upstairs"));
}

#[tokio::test]
async fn location_update_not_found() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);
    let result = repo
        .update(
            Uuid::new_v4(),
            group_id(),
            UpdateLocation {
                name: Some("X".into()),
                description: None,
                latitude: None,
                longitude: None,
            },
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn location_delete_empty() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);
    let loc = repo
        .create(
            group_id(),
            CreateLocation {
                parent_id: None,
                name: "Temp".into(),
                description: None,
                latitude: None,
                longitude: None,
            },
        )
        .await
        .unwrap();

    repo.delete(loc.id, group_id()).await.unwrap();
    assert!(repo.get(loc.id, group_id()).await.unwrap().is_none());
}

#[tokio::test]
async fn location_delete_not_found() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);
    let result = repo.delete(Uuid::new_v4(), group_id()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn location_delete_with_child_location_fails() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);
    let parent = repo
        .create(
            group_id(),
            CreateLocation {
                parent_id: None,
                name: "Parent".into(),
                description: None,
                latitude: None,
                longitude: None,
            },
        )
        .await
        .unwrap();
    repo.create(
        group_id(),
        CreateLocation {
            parent_id: Some(parent.id),
            name: "Child".into(),
            description: None,
            latitude: None,
            longitude: None,
        },
    )
    .await
    .unwrap();

    let result = repo.delete(parent.id, group_id()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn location_list_roots() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);
    repo.create(
        group_id(),
        CreateLocation {
            parent_id: None,
            name: "Room A".into(),
            description: None,
            latitude: None,
            longitude: None,
        },
    )
    .await
    .unwrap();
    let parent = repo
        .create(
            group_id(),
            CreateLocation {
                parent_id: None,
                name: "Room B".into(),
                description: None,
                latitude: None,
                longitude: None,
            },
        )
        .await
        .unwrap();
    // Child should not appear in roots
    repo.create(
        group_id(),
        CreateLocation {
            parent_id: Some(parent.id),
            name: "Shelf".into(),
            description: None,
            latitude: None,
            longitude: None,
        },
    )
    .await
    .unwrap();

    let roots = repo.list_roots(group_id()).await.unwrap();
    assert_eq!(roots.len(), 2);
}

#[tokio::test]
async fn location_list_children() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);
    let parent = repo
        .create(
            group_id(),
            CreateLocation {
                parent_id: None,
                name: "Room".into(),
                description: None,
                latitude: None,
                longitude: None,
            },
        )
        .await
        .unwrap();
    repo.create(
        group_id(),
        CreateLocation {
            parent_id: Some(parent.id),
            name: "Shelf A".into(),
            description: None,
            latitude: None,
            longitude: None,
        },
    )
    .await
    .unwrap();
    repo.create(
        group_id(),
        CreateLocation {
            parent_id: Some(parent.id),
            name: "Shelf B".into(),
            description: None,
            latitude: None,
            longitude: None,
        },
    )
    .await
    .unwrap();

    let children = repo.list_children(parent.id, group_id()).await.unwrap();
    assert_eq!(children.len(), 2);
}

#[tokio::test]
async fn location_get_tree() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);
    let room = repo
        .create(
            group_id(),
            CreateLocation {
                parent_id: None,
                name: "Room".into(),
                description: None,
                latitude: None,
                longitude: None,
            },
        )
        .await
        .unwrap();
    repo.create(
        group_id(),
        CreateLocation {
            parent_id: Some(room.id),
            name: "Shelf".into(),
            description: None,
            latitude: None,
            longitude: None,
        },
    )
    .await
    .unwrap();

    let tree = repo.get_tree(group_id()).await.unwrap();
    assert_eq!(tree.len(), 1);
    assert_eq!(tree[0].location.name, "Room");
    assert_eq!(tree[0].children.len(), 1);
    assert_eq!(tree[0].children[0].location.name, "Shelf");
}

#[tokio::test]
async fn location_has_children_with_containers() {
    let pool = test_db().await;
    let loc_repo = location_repo::SqliteLocationRepository::new(pool.clone());
    let cont_repo = container_repo::SqliteContainerRepository::new(pool);

    let loc = loc_repo
        .create(
            group_id(),
            CreateLocation {
                parent_id: None,
                name: "Room".into(),
                description: None,
                latitude: None,
                longitude: None,
            },
        )
        .await
        .unwrap();

    assert!(!loc_repo.has_children(loc.id, group_id()).await.unwrap());

    cont_repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Location(loc.id),
                name: "Box".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();

    assert!(loc_repo.has_children(loc.id, group_id()).await.unwrap());
}

// =========================================================================
// Container Repository Tests
// =========================================================================

async fn create_test_location(pool: &SqlitePool) -> Location {
    let repo = location_repo::SqliteLocationRepository::new(pool.clone());
    repo.create(
        group_id(),
        CreateLocation {
            parent_id: None,
            name: "Test Room".into(),
            description: None,
            latitude: None,
            longitude: None,
        },
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn container_create_and_get() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = container_repo::SqliteContainerRepository::new(pool);

    let cont = repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Location(loc.id),
                name: "Box 1".into(),
                description: Some("Electronics".into()),
                color: Some("blue".into()),
            },
        )
        .await
        .unwrap();

    assert_eq!(cont.name, "Box 1");
    assert_eq!(cont.parent_location_id, Some(loc.id));
    assert!(cont.parent_container_id.is_none());

    let fetched = repo.get(cont.id, group_id()).await.unwrap().unwrap();
    assert_eq!(fetched.name, "Box 1");
}

#[tokio::test]
async fn container_nested_in_container() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = container_repo::SqliteContainerRepository::new(pool);

    let parent = repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Location(loc.id),
                name: "Big Box".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();

    let child = repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Container(parent.id),
                name: "Small Box".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(child.parent_container_id, Some(parent.id));
    assert!(child.parent_location_id.is_none());
}

#[tokio::test]
async fn container_update() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = container_repo::SqliteContainerRepository::new(pool);

    let cont = repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Location(loc.id),
                name: "Box".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();

    let updated = repo
        .update(
            cont.id,
            group_id(),
            UpdateContainer {
                name: Some("Updated Box".into()),
                description: Some("New desc".into()),
                color: Some("red".into()),
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.name, "Updated Box");
    assert_eq!(updated.color.as_deref(), Some("red"));
}

#[tokio::test]
async fn container_delete_empty() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = container_repo::SqliteContainerRepository::new(pool);

    let cont = repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Location(loc.id),
                name: "Del".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();

    repo.delete(cont.id, group_id()).await.unwrap();
    assert!(repo.get(cont.id, group_id()).await.unwrap().is_none());
}

#[tokio::test]
async fn container_delete_non_empty_fails() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let cont_repo = container_repo::SqliteContainerRepository::new(pool.clone());
    let item_repo = item_repo::SqliteItemRepository::new(pool);

    let cont = cont_repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Location(loc.id),
                name: "Box".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();

    item_repo
        .create(
            group_id(),
            CreateItem {
                parent: ParentRef::Container(cont.id),
                name: "Thing".into(),
                description: None,
                aliases: None,
                keywords: None,
                category: None,
                barcode: None,
                material: None,
                color: None,
                condition_notes: None,
                quantity: None,
            },
        )
        .await
        .unwrap();

    let result = cont_repo.delete(cont.id, group_id()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn container_move_to_location() {
    let pool = test_db().await;
    let loc_repo = location_repo::SqliteLocationRepository::new(pool.clone());
    let loc1 = loc_repo
        .create(
            group_id(),
            CreateLocation {
                parent_id: None,
                name: "Room 1".into(),
                description: None,
                latitude: None,
                longitude: None,
            },
        )
        .await
        .unwrap();
    let loc2 = loc_repo
        .create(
            group_id(),
            CreateLocation {
                parent_id: None,
                name: "Room 2".into(),
                description: None,
                latitude: None,
                longitude: None,
            },
        )
        .await
        .unwrap();

    let cont_repo = container_repo::SqliteContainerRepository::new(pool);
    let cont = cont_repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Location(loc1.id),
                name: "Box".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();

    let moved = cont_repo
        .move_to(
            cont.id,
            group_id(),
            MoveTarget {
                target: ParentRef::Location(loc2.id),
            },
        )
        .await
        .unwrap();

    assert_eq!(moved.parent_location_id, Some(loc2.id));
}

#[tokio::test]
async fn container_list_by_location() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = container_repo::SqliteContainerRepository::new(pool);

    repo.create(
        group_id(),
        CreateContainer {
            parent: ParentRef::Location(loc.id),
            name: "A".into(),
            description: None,
            color: None,
        },
    )
    .await
    .unwrap();
    repo.create(
        group_id(),
        CreateContainer {
            parent: ParentRef::Location(loc.id),
            name: "B".into(),
            description: None,
            color: None,
        },
    )
    .await
    .unwrap();

    let list = repo.list_by_location(loc.id, group_id()).await.unwrap();
    assert_eq!(list.len(), 2);
}

#[tokio::test]
async fn container_ancestry() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = container_repo::SqliteContainerRepository::new(pool);

    let parent = repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Location(loc.id),
                name: "Outer".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();

    let child = repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Container(parent.id),
                name: "Inner".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();

    let ancestry = repo.get_ancestry(child.id, group_id()).await.unwrap();
    assert!(ancestry.len() >= 2);
    assert_eq!(ancestry[0].entity_type, EntityType::Location);
    assert_eq!(ancestry.last().unwrap().name, "Inner");
}

#[tokio::test]
async fn container_is_ancestor_of() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = container_repo::SqliteContainerRepository::new(pool);

    let a = repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Location(loc.id),
                name: "A".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();
    let b = repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Container(a.id),
                name: "B".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();

    assert!(repo.is_ancestor_of(a.id, b.id, group_id()).await.unwrap());
    assert!(!repo.is_ancestor_of(b.id, a.id, group_id()).await.unwrap());
}

// =========================================================================
// Item Repository Tests
// =========================================================================

#[tokio::test]
async fn item_create_and_get() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let cont_repo = container_repo::SqliteContainerRepository::new(pool.clone());
    let cont = cont_repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Location(loc.id),
                name: "Box".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();

    let item_repo = item_repo::SqliteItemRepository::new(pool);
    let item = item_repo
        .create(
            group_id(),
            CreateItem {
                parent: ParentRef::Container(cont.id),
                name: "Cable Tie".into(),
                description: Some("Reusable nylon".into()),
                aliases: Some(vec!["zip tie".into()]),
                keywords: Some(vec!["fastener".into()]),
                category: Some("Hardware".into()),
                barcode: None,
                material: Some("nylon".into()),
                color: Some("black".into()),
                condition_notes: None,
                quantity: Some(10),
            },
        )
        .await
        .unwrap();

    assert_eq!(item.name, "Cable Tie");
    assert_eq!(item.quantity, 10);
    assert_eq!(item.aliases, vec!["zip tie"]);

    let fetched = item_repo.get(item.id, group_id()).await.unwrap().unwrap();
    assert_eq!(fetched.name, "Cable Tie");
}

#[tokio::test]
async fn item_create_at_location() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = item_repo::SqliteItemRepository::new(pool);

    let item = repo
        .create(
            group_id(),
            CreateItem {
                parent: ParentRef::Location(loc.id),
                name: "Loose Item".into(),
                description: None,
                aliases: None,
                keywords: None,
                category: None,
                barcode: None,
                material: None,
                color: None,
                condition_notes: None,
                quantity: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(item.location_id, Some(loc.id));
    assert!(item.container_id.is_none());
}

#[tokio::test]
async fn item_update() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = item_repo::SqliteItemRepository::new(pool);

    let item = repo
        .create(
            group_id(),
            CreateItem {
                parent: ParentRef::Location(loc.id),
                name: "Old".into(),
                description: None,
                aliases: None,
                keywords: None,
                category: None,
                barcode: None,
                material: None,
                color: None,
                condition_notes: None,
                quantity: None,
            },
        )
        .await
        .unwrap();

    let updated = repo
        .update(
            item.id,
            group_id(),
            UpdateItem {
                name: Some("New".into()),
                description: Some("Desc".into()),
                aliases: None,
                keywords: None,
                category: None,
                barcode: None,
                material: None,
                color: None,
                condition_notes: None,
                quantity: Some(5),
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.name, "New");
    assert_eq!(updated.quantity, 5);
}

#[tokio::test]
async fn item_delete() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = item_repo::SqliteItemRepository::new(pool);

    let item = repo
        .create(
            group_id(),
            CreateItem {
                parent: ParentRef::Location(loc.id),
                name: "Del".into(),
                description: None,
                aliases: None,
                keywords: None,
                category: None,
                barcode: None,
                material: None,
                color: None,
                condition_notes: None,
                quantity: None,
            },
        )
        .await
        .unwrap();

    repo.delete(item.id, group_id()).await.unwrap();
    assert!(repo.get(item.id, group_id()).await.unwrap().is_none());
}

#[tokio::test]
async fn item_move() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let cont_repo = container_repo::SqliteContainerRepository::new(pool.clone());
    let cont = cont_repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Location(loc.id),
                name: "Box".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();

    let repo = item_repo::SqliteItemRepository::new(pool);
    let item = repo
        .create(
            group_id(),
            CreateItem {
                parent: ParentRef::Location(loc.id),
                name: "Movable".into(),
                description: None,
                aliases: None,
                keywords: None,
                category: None,
                barcode: None,
                material: None,
                color: None,
                condition_notes: None,
                quantity: None,
            },
        )
        .await
        .unwrap();

    let moved = repo
        .move_to(
            item.id,
            group_id(),
            MoveTarget {
                target: ParentRef::Container(cont.id),
            },
        )
        .await
        .unwrap();

    assert_eq!(moved.container_id, Some(cont.id));
    assert!(moved.location_id.is_none());
}

#[tokio::test]
async fn item_batch_create() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = item_repo::SqliteItemRepository::new(pool);

    let items = repo
        .create_batch(
            group_id(),
            vec![
                CreateItem {
                    parent: ParentRef::Location(loc.id),
                    name: "Item 1".into(),
                    description: None,
                    aliases: None,
                    keywords: None,
                    category: None,
                    barcode: None,
                    material: None,
                    color: None,
                    condition_notes: None,
                    quantity: None,
                },
                CreateItem {
                    parent: ParentRef::Location(loc.id),
                    name: "Item 2".into(),
                    description: None,
                    aliases: None,
                    keywords: None,
                    category: None,
                    barcode: None,
                    material: None,
                    color: None,
                    condition_notes: None,
                    quantity: None,
                },
            ],
        )
        .await
        .unwrap();

    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn item_count_by_container() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let cont_repo = container_repo::SqliteContainerRepository::new(pool.clone());
    let cont = cont_repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Location(loc.id),
                name: "Box".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();

    let repo = item_repo::SqliteItemRepository::new(pool);
    assert_eq!(
        repo.count_by_container(cont.id, group_id()).await.unwrap(),
        0
    );

    repo.create(
        group_id(),
        CreateItem {
            parent: ParentRef::Container(cont.id),
            name: "A".into(),
            description: None,
            aliases: None,
            keywords: None,
            category: None,
            barcode: None,
            material: None,
            color: None,
            condition_notes: None,
            quantity: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(
        repo.count_by_container(cont.id, group_id()).await.unwrap(),
        1
    );
}

// =========================================================================
// Photo Repository Tests
// =========================================================================

#[tokio::test]
async fn photo_create_and_get() {
    let pool = test_db().await;
    let repo = photo_repo::SqlitePhotoRepository::new(pool);

    let entity_id = Uuid::new_v4();
    let photo = repo
        .create(
            CreatePhoto {
                entity_type: EntityType::Item,
                entity_id,
                mime_type: "image/jpeg".into(),
            },
            "items/test.jpg".into(), None, None,
        )
        .await
        .unwrap();

    assert_eq!(photo.entity_type, EntityType::Item);
    assert_eq!(photo.storage_key, "items/test.jpg");
    assert!(!photo.is_primary);

    let fetched = repo.get(photo.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, photo.id);
}

#[tokio::test]
async fn photo_list_by_entity() {
    let pool = test_db().await;
    let repo = photo_repo::SqlitePhotoRepository::new(pool);
    let eid = Uuid::new_v4();

    repo.create(
        CreatePhoto {
            entity_type: EntityType::Container,
            entity_id: eid,
            mime_type: "image/png".into(),
        },
        "a.png".into(), None, None,
    )
    .await
    .unwrap();
    repo.create(
        CreatePhoto {
            entity_type: EntityType::Container,
            entity_id: eid,
            mime_type: "image/png".into(),
        },
        "b.png".into(), None, None,
    )
    .await
    .unwrap();

    let photos = repo
        .list_by_entity(EntityType::Container, eid)
        .await
        .unwrap();
    assert_eq!(photos.len(), 2);
}

#[tokio::test]
async fn photo_set_primary() {
    let pool = test_db().await;
    let repo = photo_repo::SqlitePhotoRepository::new(pool);
    let eid = Uuid::new_v4();

    let p1 = repo
        .create(
            CreatePhoto {
                entity_type: EntityType::Item,
                entity_id: eid,
                mime_type: "image/jpeg".into(),
            },
            "1.jpg".into(), None, None,
        )
        .await
        .unwrap();
    let p2 = repo
        .create(
            CreatePhoto {
                entity_type: EntityType::Item,
                entity_id: eid,
                mime_type: "image/jpeg".into(),
            },
            "2.jpg".into(), None, None,
        )
        .await
        .unwrap();

    repo.set_primary(p2.id, EntityType::Item, eid)
        .await
        .unwrap();

    let photos = repo.list_by_entity(EntityType::Item, eid).await.unwrap();
    let primary = photos.iter().find(|p| p.is_primary).unwrap();
    assert_eq!(primary.id, p2.id);
    let non_primary = photos.iter().find(|p| p.id == p1.id).unwrap();
    assert!(!non_primary.is_primary);
}

#[tokio::test]
async fn photo_delete() {
    let pool = test_db().await;
    let repo = photo_repo::SqlitePhotoRepository::new(pool);

    let photo = repo
        .create(
            CreatePhoto {
                entity_type: EntityType::Location,
                entity_id: Uuid::new_v4(),
                mime_type: "image/jpeg".into(),
            },
            "del.jpg".into(), None, None,
        )
        .await
        .unwrap();

    repo.delete(photo.id).await.unwrap();
    assert!(repo.get(photo.id).await.unwrap().is_none());
}

#[tokio::test]
async fn photo_delete_not_found() {
    let pool = test_db().await;
    let repo = photo_repo::SqlitePhotoRepository::new(pool);
    let result = repo.delete(Uuid::new_v4()).await;
    assert!(result.is_err());
}

// =========================================================================
// NFC Tag Repository Tests
// =========================================================================

#[tokio::test]
async fn nfc_tag_create_and_get() {
    let pool = test_db().await;
    let repo = nfc_tag_repo::SqliteNfcTagRepository::new(pool);

    let tag = repo
        .create(group_id(), "urn:nfc:tag:abc123".into())
        .await
        .unwrap();

    assert_eq!(tag.tag_uri, "urn:nfc:tag:abc123");
    assert!(tag.entity_type.is_none());
    assert!(tag.entity_id.is_none());

    let fetched = repo
        .get_by_uri("urn:nfc:tag:abc123")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched.id, tag.id);
}

#[tokio::test]
async fn nfc_tag_assign_and_unassign() {
    let pool = test_db().await;
    let repo = nfc_tag_repo::SqliteNfcTagRepository::new(pool);
    let entity_id = Uuid::new_v4();

    let tag = repo
        .create(group_id(), "urn:nfc:tag:xyz".into())
        .await
        .unwrap();

    let assigned = repo
        .assign(tag.id, EntityType::Container, entity_id)
        .await
        .unwrap();

    assert_eq!(assigned.entity_type, Some(EntityType::Container));
    assert_eq!(assigned.entity_id, Some(entity_id));
    assert!(assigned.assigned_at.is_some());

    repo.unassign(tag.id).await.unwrap();
    let after = repo.get_by_uri("urn:nfc:tag:xyz").await.unwrap().unwrap();
    assert!(after.entity_type.is_none());
    assert!(after.entity_id.is_none());
}

#[tokio::test]
async fn nfc_tag_delete() {
    let pool = test_db().await;
    let repo = nfc_tag_repo::SqliteNfcTagRepository::new(pool);

    let tag = repo
        .create(group_id(), "urn:nfc:tag:del".into())
        .await
        .unwrap();

    repo.delete(tag.id).await.unwrap();
    assert!(repo.get_by_uri("urn:nfc:tag:del").await.unwrap().is_none());
}

// =========================================================================
// Search Repository Tests
// =========================================================================

#[tokio::test]
async fn search_index_and_find() {
    let pool = test_db().await;
    let repo = search_repo::SqliteSearchRepository::new(pool);
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    repo.index(
        EntityType::Item,
        id1,
        group_id(),
        "cable tie nylon fastener",
    )
    .await
    .unwrap();
    repo.index(EntityType::Item, id2, group_id(), "rubber band elastic")
        .await
        .unwrap();

    let results = repo.search(group_id(), "cable", 10).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entity_id, id1);
}

#[tokio::test]
async fn search_remove() {
    let pool = test_db().await;
    let repo = search_repo::SqliteSearchRepository::new(pool);
    let id = Uuid::new_v4();

    repo.index(EntityType::Item, id, group_id(), "unique searchterm")
        .await
        .unwrap();

    let results = repo.search(group_id(), "searchterm", 10).await.unwrap();
    assert_eq!(results.len(), 1);

    repo.remove(EntityType::Item, id).await.unwrap();

    let results = repo.search(group_id(), "searchterm", 10).await.unwrap();
    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn search_empty_results() {
    let pool = test_db().await;
    let repo = search_repo::SqliteSearchRepository::new(pool);

    let results = repo.search(group_id(), "nonexistent", 10).await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn search_group_isolation() {
    let pool = test_db().await;
    let repo = search_repo::SqliteSearchRepository::new(pool);
    let other_group = Uuid::new_v4();

    repo.index(EntityType::Item, Uuid::new_v4(), other_group, "secret item")
        .await
        .unwrap();

    let results = repo.search(group_id(), "secret", 10).await.unwrap();
    assert!(results.is_empty());
}

// =========================================================================
// User Repository Tests
// =========================================================================

#[tokio::test]
async fn user_upsert_and_get() {
    let pool = test_db().await;
    let repo = user_repo::SqliteUserRepository::new(pool);

    let user = repo
        .upsert_by_external_id(CreateUser {
            external_id: "ext-123".into(),
            email: "alice@example.com".into(),
            display_name: "Alice".into(),
        })
        .await
        .unwrap();

    assert_eq!(user.external_id, "ext-123");
    assert_eq!(user.email, "alice@example.com");

    let fetched = repo.get(user.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, user.id);

    let by_ext = repo.get_by_external_id("ext-123").await.unwrap().unwrap();
    assert_eq!(by_ext.id, user.id);
}

#[tokio::test]
async fn user_upsert_updates_existing() {
    let pool = test_db().await;
    let repo = user_repo::SqliteUserRepository::new(pool);

    let user1 = repo
        .upsert_by_external_id(CreateUser {
            external_id: "ext-dup".into(),
            email: "old@example.com".into(),
            display_name: "Old Name".into(),
        })
        .await
        .unwrap();

    let user2 = repo
        .upsert_by_external_id(CreateUser {
            external_id: "ext-dup".into(),
            email: "new@example.com".into(),
            display_name: "New Name".into(),
        })
        .await
        .unwrap();

    // Same user, updated fields
    assert_eq!(user1.id, user2.id);
    assert_eq!(user2.email, "new@example.com");
    assert_eq!(user2.display_name, "New Name");
}

#[tokio::test]
async fn user_get_not_found() {
    let pool = test_db().await;
    let repo = user_repo::SqliteUserRepository::new(pool);
    assert!(repo.get(Uuid::new_v4()).await.unwrap().is_none());
}

#[tokio::test]
async fn user_get_by_external_id_not_found() {
    let pool = test_db().await;
    let repo = user_repo::SqliteUserRepository::new(pool);
    assert!(
        repo.get_by_external_id("nonexistent")
            .await
            .unwrap()
            .is_none()
    );
}

// =========================================================================
// Group Repository Tests
// =========================================================================

#[tokio::test]
async fn group_get_or_create() {
    let pool = test_db().await;
    let repo = group_repo::SqliteGroupRepository::new(pool);

    let g1 = repo.get_or_create_by_name("family").await.unwrap();
    assert_eq!(g1.name, "family");

    // Idempotent: same group returned
    let g2 = repo.get_or_create_by_name("family").await.unwrap();
    assert_eq!(g1.id, g2.id);
}

#[tokio::test]
async fn group_get_by_id() {
    let pool = test_db().await;
    let repo = group_repo::SqliteGroupRepository::new(pool);

    let created = repo.get_or_create_by_name("test-group").await.unwrap();
    let fetched = repo.get(created.id).await.unwrap().unwrap();
    assert_eq!(fetched.name, "test-group");
}

#[tokio::test]
async fn group_get_not_found() {
    let pool = test_db().await;
    let repo = group_repo::SqliteGroupRepository::new(pool);
    assert!(repo.get(Uuid::new_v4()).await.unwrap().is_none());
}

#[tokio::test]
async fn group_default_seeded() {
    let pool = test_db().await;
    let repo = group_repo::SqliteGroupRepository::new(pool);

    // The migration seeds the default group
    let default_group = repo.get(group_id()).await.unwrap().unwrap();
    assert_eq!(default_group.name, "default");
}

// =========================================================================
// UserGroup Repository Tests
// =========================================================================

async fn create_test_user(pool: &SqlitePool) -> storeit_domain::entities::User {
    let repo = user_repo::SqliteUserRepository::new(pool.clone());
    repo.upsert_by_external_id(CreateUser {
        external_id: format!("ext-{}", Uuid::new_v4()),
        email: "test@example.com".into(),
        display_name: "Test".into(),
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn user_group_set_and_list() {
    let pool = test_db().await;
    let ug_repo = user_group_repo::SqliteUserGroupRepository::new(pool.clone());
    let group_repo = group_repo::SqliteGroupRepository::new(pool.clone());

    let user = create_test_user(&pool).await;
    let g1 = group_repo.get_or_create_by_name("group-a").await.unwrap();
    let g2 = group_repo.get_or_create_by_name("group-b").await.unwrap();

    ug_repo
        .set_memberships(
            user.id,
            vec![(g1.id, GroupRole::Owner), (g2.id, GroupRole::Member)],
        )
        .await
        .unwrap();

    let groups = ug_repo.list_groups_for_user(user.id).await.unwrap();
    assert_eq!(groups.len(), 2);

    let (ga, role_a) = groups.iter().find(|(g, _)| g.name == "group-a").unwrap();
    assert_eq!(ga.id, g1.id);
    assert_eq!(*role_a, GroupRole::Owner);
}

#[tokio::test]
async fn user_group_is_member() {
    let pool = test_db().await;
    let ug_repo = user_group_repo::SqliteUserGroupRepository::new(pool.clone());
    let g_repo = group_repo::SqliteGroupRepository::new(pool.clone());

    let user = create_test_user(&pool).await;
    let group = g_repo.get_or_create_by_name("members-test").await.unwrap();

    assert!(!ug_repo.is_member(user.id, group.id).await.unwrap());

    ug_repo
        .set_memberships(user.id, vec![(group.id, GroupRole::Member)])
        .await
        .unwrap();

    assert!(ug_repo.is_member(user.id, group.id).await.unwrap());
}

#[tokio::test]
async fn user_group_set_replaces() {
    let pool = test_db().await;
    let ug_repo = user_group_repo::SqliteUserGroupRepository::new(pool.clone());
    let g_repo = group_repo::SqliteGroupRepository::new(pool.clone());

    let user = create_test_user(&pool).await;
    let g1 = g_repo.get_or_create_by_name("replace-a").await.unwrap();
    let g2 = g_repo.get_or_create_by_name("replace-b").await.unwrap();

    // Set initial memberships
    ug_repo
        .set_memberships(user.id, vec![(g1.id, GroupRole::Member)])
        .await
        .unwrap();
    assert_eq!(
        ug_repo.list_groups_for_user(user.id).await.unwrap().len(),
        1
    );

    // Replace with new set
    ug_repo
        .set_memberships(user.id, vec![(g2.id, GroupRole::Owner)])
        .await
        .unwrap();

    let groups = ug_repo.list_groups_for_user(user.id).await.unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].0.name, "replace-b");
    assert_eq!(groups[0].1, GroupRole::Owner);
}

// =========================================================================
// Session Repository Tests
// =========================================================================

#[tokio::test]
async fn session_create_and_get() {
    let pool = test_db().await;
    let session_repo = session_repo::SqliteSessionRepository::new(pool.clone());

    let user = create_test_user(&pool).await;

    let session = session_repo
        .create(Session {
            id: "sess-test-123".into(),
            user_id: user.id,
            active_group_id: group_id(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
            created_at: chrono::Utc::now(),
        })
        .await
        .unwrap();

    assert_eq!(session.id, "sess-test-123");
    assert_eq!(session.user_id, user.id);

    let fetched = session_repo.get("sess-test-123").await.unwrap().unwrap();
    assert_eq!(fetched.user_id, user.id);
}

#[tokio::test]
async fn session_get_not_found() {
    let pool = test_db().await;
    let repo = session_repo::SqliteSessionRepository::new(pool);
    assert!(repo.get("nonexistent").await.unwrap().is_none());
}

#[tokio::test]
async fn session_update_active_group() {
    let pool = test_db().await;
    let s_repo = session_repo::SqliteSessionRepository::new(pool.clone());
    let g_repo = group_repo::SqliteGroupRepository::new(pool.clone());

    let user = create_test_user(&pool).await;
    let new_group = g_repo.get_or_create_by_name("new-group").await.unwrap();

    s_repo
        .create(Session {
            id: "sess-switch".into(),
            user_id: user.id,
            active_group_id: group_id(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
            created_at: chrono::Utc::now(),
        })
        .await
        .unwrap();

    let updated = s_repo
        .update_active_group("sess-switch", new_group.id)
        .await
        .unwrap();

    assert_eq!(updated.active_group_id, new_group.id);
}

#[tokio::test]
async fn session_delete() {
    let pool = test_db().await;
    let s_repo = session_repo::SqliteSessionRepository::new(pool.clone());

    let user = create_test_user(&pool).await;

    s_repo
        .create(Session {
            id: "sess-del".into(),
            user_id: user.id,
            active_group_id: group_id(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
            created_at: chrono::Utc::now(),
        })
        .await
        .unwrap();

    s_repo.delete("sess-del").await.unwrap();
    assert!(s_repo.get("sess-del").await.unwrap().is_none());
}

#[tokio::test]
async fn session_delete_expired() {
    let pool = test_db().await;
    let s_repo = session_repo::SqliteSessionRepository::new(pool.clone());

    let user = create_test_user(&pool).await;

    // Create an already-expired session
    s_repo
        .create(Session {
            id: "sess-expired".into(),
            user_id: user.id,
            active_group_id: group_id(),
            expires_at: chrono::Utc::now() - chrono::Duration::hours(1),
            created_at: chrono::Utc::now(),
        })
        .await
        .unwrap();

    // Create a valid session
    s_repo
        .create(Session {
            id: "sess-valid".into(),
            user_id: user.id,
            active_group_id: group_id(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
            created_at: chrono::Utc::now(),
        })
        .await
        .unwrap();

    let deleted = s_repo.delete_expired().await.unwrap();
    assert_eq!(deleted, 1);

    // Expired is gone, valid remains
    assert!(s_repo.get("sess-expired").await.unwrap().is_none());
    assert!(s_repo.get("sess-valid").await.unwrap().is_some());
}

// =========================================================================
// SqliteDb Tests
// =========================================================================

#[tokio::test]
async fn sqlite_db_migrate() {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect");
    let db = SqliteDb::new(pool.clone());
    db.migrate().await.expect("migrations should succeed");

    // Verify a table exists
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='locations'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(result.0, 1);
}

// =========================================================================
// Additional Location Repository Tests
// =========================================================================

#[tokio::test]
async fn location_has_children_with_child_locations() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);

    let parent = repo
        .create(
            group_id(),
            CreateLocation {
                parent_id: None,
                name: "Parent".into(),
                description: None,
                latitude: None,
                longitude: None,
            },
        )
        .await
        .unwrap();

    assert!(!repo.has_children(parent.id, group_id()).await.unwrap());

    repo.create(
        group_id(),
        CreateLocation {
            parent_id: Some(parent.id),
            name: "Child".into(),
            description: None,
            latitude: None,
            longitude: None,
        },
    )
    .await
    .unwrap();

    assert!(repo.has_children(parent.id, group_id()).await.unwrap());
}

#[tokio::test]
async fn location_has_children_with_items() {
    let pool = test_db().await;
    let loc_repo = location_repo::SqliteLocationRepository::new(pool.clone());
    let i_repo = item_repo::SqliteItemRepository::new(pool);

    let loc = loc_repo
        .create(
            group_id(),
            CreateLocation {
                parent_id: None,
                name: "Room".into(),
                description: None,
                latitude: None,
                longitude: None,
            },
        )
        .await
        .unwrap();

    assert!(!loc_repo.has_children(loc.id, group_id()).await.unwrap());

    i_repo
        .create(
            group_id(),
            CreateItem {
                parent: ParentRef::Location(loc.id),
                name: "Loose Item".into(),
                description: None,
                aliases: None,
                keywords: None,
                category: None,
                barcode: None,
                material: None,
                color: None,
                condition_notes: None,
                quantity: None,
            },
        )
        .await
        .unwrap();

    assert!(loc_repo.has_children(loc.id, group_id()).await.unwrap());
}

#[tokio::test]
async fn location_list_all_unscoped() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);

    repo.create(
        group_id(),
        CreateLocation {
            parent_id: None,
            name: "A".into(),
            description: None,
            latitude: None,
            longitude: None,
        },
    )
    .await
    .unwrap();

    // list_all_unscoped is not group-scoped, but we can only create with valid groups.
    // At minimum, verify the one we created shows up.
    let all = repo.list_all_unscoped().await.unwrap();
    assert!(!all.is_empty());
    assert!(all.iter().any(|l| l.name == "A"));
}

#[tokio::test]
async fn location_insert_raw_and_get() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);

    let now = chrono::Utc::now();
    let loc = Location {
        id: Uuid::new_v4(),
        group_id: group_id(),
        parent_id: None,
        name: "Raw Location".into(),
        description: Some("Inserted raw".into()),
        latitude: Some(10.0),
        longitude: Some(20.0),
        created_at: now,
        updated_at: now,
    };

    repo.insert_raw(&loc).await.unwrap();

    let fetched = repo.get(loc.id, group_id()).await.unwrap().unwrap();
    assert_eq!(fetched.name, "Raw Location");
    assert_eq!(fetched.description.as_deref(), Some("Inserted raw"));
    assert!((fetched.latitude.unwrap() - 10.0).abs() < 1e-6);
}

#[tokio::test]
async fn location_delete_all() {
    let pool = test_db().await;
    let repo = location_repo::SqliteLocationRepository::new(pool);

    repo.create(
        group_id(),
        CreateLocation {
            parent_id: None,
            name: "A".into(),
            description: None,
            latitude: None,
            longitude: None,
        },
    )
    .await
    .unwrap();
    repo.create(
        group_id(),
        CreateLocation {
            parent_id: None,
            name: "B".into(),
            description: None,
            latitude: None,
            longitude: None,
        },
    )
    .await
    .unwrap();

    repo.delete_all().await.unwrap();

    let all = repo.list_all_unscoped().await.unwrap();
    assert!(all.is_empty());
}

// =========================================================================
// Additional Container Repository Tests
// =========================================================================

#[tokio::test]
async fn container_list_by_container() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = container_repo::SqliteContainerRepository::new(pool);

    let parent = repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Location(loc.id),
                name: "Outer".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();

    repo.create(
        group_id(),
        CreateContainer {
            parent: ParentRef::Container(parent.id),
            name: "Inner A".into(),
            description: None,
            color: None,
        },
    )
    .await
    .unwrap();
    repo.create(
        group_id(),
        CreateContainer {
            parent: ParentRef::Container(parent.id),
            name: "Inner B".into(),
            description: None,
            color: None,
        },
    )
    .await
    .unwrap();

    let nested = repo.list_by_container(parent.id, group_id()).await.unwrap();
    assert_eq!(nested.len(), 2);
}

#[tokio::test]
async fn container_list_all() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = container_repo::SqliteContainerRepository::new(pool);

    repo.create(
        group_id(),
        CreateContainer {
            parent: ParentRef::Location(loc.id),
            name: "Box 1".into(),
            description: None,
            color: None,
        },
    )
    .await
    .unwrap();
    repo.create(
        group_id(),
        CreateContainer {
            parent: ParentRef::Location(loc.id),
            name: "Box 2".into(),
            description: None,
            color: None,
        },
    )
    .await
    .unwrap();

    let all = repo.list_all(group_id()).await.unwrap();
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn container_has_children_with_nested_container() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = container_repo::SqliteContainerRepository::new(pool);

    let parent = repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Location(loc.id),
                name: "Parent Box".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();

    assert!(!repo.has_children(parent.id, group_id()).await.unwrap());

    repo.create(
        group_id(),
        CreateContainer {
            parent: ParentRef::Container(parent.id),
            name: "Child Box".into(),
            description: None,
            color: None,
        },
    )
    .await
    .unwrap();

    assert!(repo.has_children(parent.id, group_id()).await.unwrap());
}

#[tokio::test]
async fn container_has_children_with_items() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let cont_repo = container_repo::SqliteContainerRepository::new(pool.clone());
    let i_repo = item_repo::SqliteItemRepository::new(pool);

    let cont = cont_repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Location(loc.id),
                name: "Box".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();

    assert!(!cont_repo.has_children(cont.id, group_id()).await.unwrap());

    i_repo
        .create(
            group_id(),
            CreateItem {
                parent: ParentRef::Container(cont.id),
                name: "Widget".into(),
                description: None,
                aliases: None,
                keywords: None,
                category: None,
                barcode: None,
                material: None,
                color: None,
                condition_notes: None,
                quantity: None,
            },
        )
        .await
        .unwrap();

    assert!(cont_repo.has_children(cont.id, group_id()).await.unwrap());
}

#[tokio::test]
async fn container_list_all_unscoped() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = container_repo::SqliteContainerRepository::new(pool);

    repo.create(
        group_id(),
        CreateContainer {
            parent: ParentRef::Location(loc.id),
            name: "Unscoped Box".into(),
            description: None,
            color: None,
        },
    )
    .await
    .unwrap();

    let all = repo.list_all_unscoped().await.unwrap();
    assert!(!all.is_empty());
    assert!(all.iter().any(|c| c.name == "Unscoped Box"));
}

#[tokio::test]
async fn container_insert_raw_and_get() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = container_repo::SqliteContainerRepository::new(pool);

    let now = chrono::Utc::now();
    let cont = Container {
        id: Uuid::new_v4(),
        group_id: group_id(),
        parent_location_id: Some(loc.id),
        parent_container_id: None,
        name: "Raw Container".into(),
        description: Some("Inserted raw".into()),
        color: Some("green".into()),
        created_at: now,
        updated_at: now,
    };

    repo.insert_raw(&cont).await.unwrap();

    let fetched = repo.get(cont.id, group_id()).await.unwrap().unwrap();
    assert_eq!(fetched.name, "Raw Container");
    assert_eq!(fetched.color.as_deref(), Some("green"));
}

#[tokio::test]
async fn container_delete_all() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = container_repo::SqliteContainerRepository::new(pool);

    repo.create(
        group_id(),
        CreateContainer {
            parent: ParentRef::Location(loc.id),
            name: "A".into(),
            description: None,
            color: None,
        },
    )
    .await
    .unwrap();

    repo.delete_all().await.unwrap();

    let all = repo.list_all(group_id()).await.unwrap();
    assert!(all.is_empty());
}

// =========================================================================
// Additional Item Repository Tests
// =========================================================================

#[tokio::test]
async fn item_list_by_container() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let cont_repo = container_repo::SqliteContainerRepository::new(pool.clone());
    let cont = cont_repo
        .create(
            group_id(),
            CreateContainer {
                parent: ParentRef::Location(loc.id),
                name: "Box".into(),
                description: None,
                color: None,
            },
        )
        .await
        .unwrap();

    let repo = item_repo::SqliteItemRepository::new(pool);
    repo.create(
        group_id(),
        CreateItem {
            parent: ParentRef::Container(cont.id),
            name: "Item A".into(),
            description: None,
            aliases: None,
            keywords: None,
            category: None,
            barcode: None,
            material: None,
            color: None,
            condition_notes: None,
            quantity: None,
        },
    )
    .await
    .unwrap();
    repo.create(
        group_id(),
        CreateItem {
            parent: ParentRef::Container(cont.id),
            name: "Item B".into(),
            description: None,
            aliases: None,
            keywords: None,
            category: None,
            barcode: None,
            material: None,
            color: None,
            condition_notes: None,
            quantity: None,
        },
    )
    .await
    .unwrap();

    let items = repo.list_by_container(cont.id, group_id()).await.unwrap();
    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn item_list_by_location() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = item_repo::SqliteItemRepository::new(pool);

    repo.create(
        group_id(),
        CreateItem {
            parent: ParentRef::Location(loc.id),
            name: "Loose A".into(),
            description: None,
            aliases: None,
            keywords: None,
            category: None,
            barcode: None,
            material: None,
            color: None,
            condition_notes: None,
            quantity: None,
        },
    )
    .await
    .unwrap();
    repo.create(
        group_id(),
        CreateItem {
            parent: ParentRef::Location(loc.id),
            name: "Loose B".into(),
            description: None,
            aliases: None,
            keywords: None,
            category: None,
            barcode: None,
            material: None,
            color: None,
            condition_notes: None,
            quantity: None,
        },
    )
    .await
    .unwrap();

    let items = repo.list_by_location(loc.id, group_id()).await.unwrap();
    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn item_list_all() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = item_repo::SqliteItemRepository::new(pool);

    repo.create(
        group_id(),
        CreateItem {
            parent: ParentRef::Location(loc.id),
            name: "Item 1".into(),
            description: None,
            aliases: None,
            keywords: None,
            category: None,
            barcode: None,
            material: None,
            color: None,
            condition_notes: None,
            quantity: None,
        },
    )
    .await
    .unwrap();
    repo.create(
        group_id(),
        CreateItem {
            parent: ParentRef::Location(loc.id),
            name: "Item 2".into(),
            description: None,
            aliases: None,
            keywords: None,
            category: None,
            barcode: None,
            material: None,
            color: None,
            condition_notes: None,
            quantity: None,
        },
    )
    .await
    .unwrap();

    let all = repo.list_all(group_id()).await.unwrap();
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn item_count_by_location() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = item_repo::SqliteItemRepository::new(pool);

    assert_eq!(repo.count_by_location(loc.id, group_id()).await.unwrap(), 0);

    repo.create(
        group_id(),
        CreateItem {
            parent: ParentRef::Location(loc.id),
            name: "A".into(),
            description: None,
            aliases: None,
            keywords: None,
            category: None,
            barcode: None,
            material: None,
            color: None,
            condition_notes: None,
            quantity: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(repo.count_by_location(loc.id, group_id()).await.unwrap(), 1);
}

#[tokio::test]
async fn item_list_all_unscoped() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = item_repo::SqliteItemRepository::new(pool);

    repo.create(
        group_id(),
        CreateItem {
            parent: ParentRef::Location(loc.id),
            name: "Unscoped Item".into(),
            description: None,
            aliases: None,
            keywords: None,
            category: None,
            barcode: None,
            material: None,
            color: None,
            condition_notes: None,
            quantity: None,
        },
    )
    .await
    .unwrap();

    let all = repo.list_all_unscoped().await.unwrap();
    assert!(!all.is_empty());
    assert!(all.iter().any(|i| i.name == "Unscoped Item"));
}

#[tokio::test]
async fn item_insert_raw_and_get() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = item_repo::SqliteItemRepository::new(pool);

    let now = chrono::Utc::now();
    let item = Item {
        id: Uuid::new_v4(),
        group_id: group_id(),
        container_id: None,
        location_id: Some(loc.id),
        name: "Raw Item".into(),
        description: Some("Inserted raw".into()),
        aliases: vec!["alias1".into()],
        keywords: vec!["kw1".into()],
        category: Some("Tools".into()),
        barcode: None,
        material: None,
        color: None,
        condition_notes: None,
        quantity: 3,
        ai_raw: None,
        created_at: now,
        updated_at: now,
    };

    repo.insert_raw(&item).await.unwrap();

    let fetched = repo.get(item.id, group_id()).await.unwrap().unwrap();
    assert_eq!(fetched.name, "Raw Item");
    assert_eq!(fetched.quantity, 3);
}

#[tokio::test]
async fn item_delete_all() {
    let pool = test_db().await;
    let loc = create_test_location(&pool).await;
    let repo = item_repo::SqliteItemRepository::new(pool);

    repo.create(
        group_id(),
        CreateItem {
            parent: ParentRef::Location(loc.id),
            name: "A".into(),
            description: None,
            aliases: None,
            keywords: None,
            category: None,
            barcode: None,
            material: None,
            color: None,
            condition_notes: None,
            quantity: None,
        },
    )
    .await
    .unwrap();

    repo.delete_all().await.unwrap();

    let all = repo.list_all(group_id()).await.unwrap();
    assert!(all.is_empty());
}

// =========================================================================
// Additional Photo Repository Tests
// =========================================================================

#[tokio::test]
async fn photo_count_by_storage_key() {
    let pool = test_db().await;
    let repo = photo_repo::SqlitePhotoRepository::new(pool);
    let eid = Uuid::new_v4();

    assert_eq!(repo.count_by_storage_key("shared.jpg").await.unwrap(), 0);

    repo.create(
        CreatePhoto {
            entity_type: EntityType::Item,
            entity_id: eid,
            mime_type: "image/jpeg".into(),
        },
        "shared.jpg".into(), None, None,
    )
    .await
    .unwrap();

    assert_eq!(repo.count_by_storage_key("shared.jpg").await.unwrap(), 1);

    repo.create(
        CreatePhoto {
            entity_type: EntityType::Item,
            entity_id: Uuid::new_v4(),
            mime_type: "image/jpeg".into(),
        },
        "shared.jpg".into(), None, None,
    )
    .await
    .unwrap();

    assert_eq!(repo.count_by_storage_key("shared.jpg").await.unwrap(), 2);
}

#[tokio::test]
async fn photo_update_storage_key() {
    let pool = test_db().await;
    let repo = photo_repo::SqlitePhotoRepository::new(pool);

    let photo = repo
        .create(
            CreatePhoto {
                entity_type: EntityType::Item,
                entity_id: Uuid::new_v4(),
                mime_type: "image/jpeg".into(),
            },
            "old-key.jpg".into(), None, None,
        )
        .await
        .unwrap();

    assert_eq!(photo.storage_key, "old-key.jpg");

    repo.update_storage_key(photo.id, "new-key.jpg")
        .await
        .unwrap();

    let fetched = repo.get(photo.id).await.unwrap().unwrap();
    assert_eq!(fetched.storage_key, "new-key.jpg");
}

#[tokio::test]
async fn photo_list_all() {
    let pool = test_db().await;
    let repo = photo_repo::SqlitePhotoRepository::new(pool);

    repo.create(
        CreatePhoto {
            entity_type: EntityType::Item,
            entity_id: Uuid::new_v4(),
            mime_type: "image/jpeg".into(),
        },
        "1.jpg".into(), None, None,
    )
    .await
    .unwrap();
    repo.create(
        CreatePhoto {
            entity_type: EntityType::Container,
            entity_id: Uuid::new_v4(),
            mime_type: "image/png".into(),
        },
        "2.png".into(), None, None,
    )
    .await
    .unwrap();

    let all = repo.list_all().await.unwrap();
    assert_eq!(all.len(), 2);
}

#[tokio::test]
async fn photo_insert_raw_and_get() {
    let pool = test_db().await;
    let repo = photo_repo::SqlitePhotoRepository::new(pool);

    let now = chrono::Utc::now();
    let photo = Photo {
        id: Uuid::new_v4(),
        entity_type: EntityType::Location,
        entity_id: Uuid::new_v4(),
        storage_key: "raw/photo.jpg".into(),
        thumbnail_key: Some("raw/photo_thumb.webp".into()),
        large_key: Some("raw/photo_large.webp".into()),
        mime_type: "image/jpeg".into(),
        is_primary: true,
        rotation: 0,
        created_at: now,
    };

    repo.insert_raw(&photo).await.unwrap();

    let fetched = repo.get(photo.id).await.unwrap().unwrap();
    assert_eq!(fetched.storage_key, "raw/photo.jpg");
    assert!(fetched.is_primary);
}

#[tokio::test]
async fn photo_delete_all() {
    let pool = test_db().await;
    let repo = photo_repo::SqlitePhotoRepository::new(pool);

    repo.create(
        CreatePhoto {
            entity_type: EntityType::Item,
            entity_id: Uuid::new_v4(),
            mime_type: "image/jpeg".into(),
        },
        "a.jpg".into(), None, None,
    )
    .await
    .unwrap();

    repo.delete_all().await.unwrap();

    let all = repo.list_all().await.unwrap();
    assert!(all.is_empty());
}

// =========================================================================
// Additional NFC Tag Repository Tests
// =========================================================================

#[tokio::test]
async fn nfc_tag_get_by_id() {
    let pool = test_db().await;
    let repo = nfc_tag_repo::SqliteNfcTagRepository::new(pool);

    let tag = repo
        .create(group_id(), "urn:nfc:tag:getid".into())
        .await
        .unwrap();

    let fetched = repo.get(tag.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, tag.id);
    assert_eq!(fetched.tag_uri, "urn:nfc:tag:getid");
}

#[tokio::test]
async fn nfc_tag_get_by_id_not_found() {
    let pool = test_db().await;
    let repo = nfc_tag_repo::SqliteNfcTagRepository::new(pool);
    assert!(repo.get(Uuid::new_v4()).await.unwrap().is_none());
}

#[tokio::test]
async fn nfc_tag_list_by_group() {
    let pool = test_db().await;
    let repo = nfc_tag_repo::SqliteNfcTagRepository::new(pool);

    repo.create(group_id(), "urn:nfc:tag:g1".into())
        .await
        .unwrap();
    repo.create(group_id(), "urn:nfc:tag:g2".into())
        .await
        .unwrap();

    let tags = repo.list_by_group(group_id()).await.unwrap();
    assert_eq!(tags.len(), 2);

    // Different group should have none
    let other = repo.list_by_group(Uuid::new_v4()).await.unwrap();
    assert!(other.is_empty());
}

#[tokio::test]
async fn nfc_tag_list_by_entity() {
    let pool = test_db().await;
    let repo = nfc_tag_repo::SqliteNfcTagRepository::new(pool);
    let entity_id = Uuid::new_v4();

    let tag1 = repo
        .create(group_id(), "urn:nfc:tag:e1".into())
        .await
        .unwrap();
    let tag2 = repo
        .create(group_id(), "urn:nfc:tag:e2".into())
        .await
        .unwrap();
    repo.create(group_id(), "urn:nfc:tag:e3".into())
        .await
        .unwrap();

    repo.assign(tag1.id, EntityType::Container, entity_id)
        .await
        .unwrap();
    repo.assign(tag2.id, EntityType::Container, entity_id)
        .await
        .unwrap();

    let tags = repo
        .list_by_entity(EntityType::Container, entity_id)
        .await
        .unwrap();
    assert_eq!(tags.len(), 2);

    // Unrelated entity should have none
    let none = repo
        .list_by_entity(EntityType::Item, Uuid::new_v4())
        .await
        .unwrap();
    assert!(none.is_empty());
}

#[tokio::test]
async fn nfc_tag_list_all_unscoped() {
    let pool = test_db().await;
    let repo = nfc_tag_repo::SqliteNfcTagRepository::new(pool);

    repo.create(group_id(), "urn:nfc:tag:u1".into())
        .await
        .unwrap();

    let all = repo.list_all_unscoped().await.unwrap();
    assert!(!all.is_empty());
    assert!(all.iter().any(|t| t.tag_uri == "urn:nfc:tag:u1"));
}

#[tokio::test]
async fn nfc_tag_insert_raw_and_get() {
    let pool = test_db().await;
    let repo = nfc_tag_repo::SqliteNfcTagRepository::new(pool);

    let now = chrono::Utc::now();
    let tag = NfcTag {
        id: Uuid::new_v4(),
        group_id: group_id(),
        tag_uri: "urn:nfc:tag:raw".into(),
        entity_type: Some(EntityType::Container),
        entity_id: Some(Uuid::new_v4()),
        created_at: now,
        assigned_at: Some(now),
    };

    repo.insert_raw(&tag).await.unwrap();

    let fetched = repo.get(tag.id).await.unwrap().unwrap();
    assert_eq!(fetched.tag_uri, "urn:nfc:tag:raw");
    assert_eq!(fetched.entity_type, Some(EntityType::Container));
}

#[tokio::test]
async fn nfc_tag_delete_all() {
    let pool = test_db().await;
    let repo = nfc_tag_repo::SqliteNfcTagRepository::new(pool);

    repo.create(group_id(), "urn:nfc:tag:da1".into())
        .await
        .unwrap();

    repo.delete_all().await.unwrap();

    let all = repo.list_all_unscoped().await.unwrap();
    assert!(all.is_empty());
}

// =========================================================================
// Additional Search Repository Tests
// =========================================================================

#[tokio::test]
async fn search_rebuild_index() {
    let pool = test_db().await;
    let repo = search_repo::SqliteSearchRepository::new(pool);
    let id = Uuid::new_v4();

    repo.index(EntityType::Item, id, group_id(), "rebuild test item")
        .await
        .unwrap();

    // rebuild_index should succeed without error
    repo.rebuild_index().await.unwrap();

    // Data should still be searchable (or cleared depending on implementation).
    // At minimum, calling rebuild_index should not panic.
}

// =========================================================================
// Additional User Repository Tests
// =========================================================================

#[tokio::test]
async fn user_create_local() {
    let pool = test_db().await;
    let repo = user_repo::SqliteUserRepository::new(pool);

    let user = repo
        .create_local(CreateLocalUser {
            username: "localuser".into(),
            email: "local@example.com".into(),
            display_name: "Local User".into(),
            password_hash: "$argon2id$hash".into(),
            is_admin: false,
        })
        .await
        .unwrap();

    assert_eq!(user.email, "local@example.com");
    assert_eq!(user.display_name, "Local User");
    assert!(!user.is_admin);

    // Local users have external_id prefixed with "local:"
    let by_ext = repo
        .get_by_external_id("local:localuser")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(by_ext.id, user.id);
}

#[tokio::test]
async fn user_get_password_hash() {
    let pool = test_db().await;
    let repo = user_repo::SqliteUserRepository::new(pool);

    repo.create_local(CreateLocalUser {
        username: "hashuser".into(),
        email: "hash@example.com".into(),
        display_name: "Hash".into(),
        password_hash: "$argon2id$testhash".into(),
        is_admin: false,
    })
    .await
    .unwrap();

    let hash = repo
        .get_password_hash("local:hashuser")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(hash, "$argon2id$testhash");

    // Non-existent user returns None
    let no_hash = repo.get_password_hash("nobody").await.unwrap();
    assert!(no_hash.is_none());
}

#[tokio::test]
async fn user_set_password_hash() {
    let pool = test_db().await;
    let repo = user_repo::SqliteUserRepository::new(pool);

    let user = repo
        .create_local(CreateLocalUser {
            username: "setpw".into(),
            email: "setpw@example.com".into(),
            display_name: "Set PW".into(),
            password_hash: "oldhash".into(),
            is_admin: false,
        })
        .await
        .unwrap();

    repo.set_password_hash(user.id, "newhash").await.unwrap();

    let hash = repo
        .get_password_hash("local:setpw")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(hash, "newhash");
}

#[tokio::test]
async fn user_set_admin() {
    let pool = test_db().await;
    let repo = user_repo::SqliteUserRepository::new(pool);

    let user = repo
        .upsert_by_external_id(CreateUser {
            external_id: "ext-admin".into(),
            email: "admin@example.com".into(),
            display_name: "Admin".into(),
        })
        .await
        .unwrap();

    assert!(!user.is_admin);

    repo.set_admin(user.id, true).await.unwrap();

    let fetched = repo.get(user.id).await.unwrap().unwrap();
    assert!(fetched.is_admin);

    repo.set_admin(user.id, false).await.unwrap();
    let fetched = repo.get(user.id).await.unwrap().unwrap();
    assert!(!fetched.is_admin);
}

#[tokio::test]
async fn user_list_all() {
    let pool = test_db().await;
    let repo = user_repo::SqliteUserRepository::new(pool);

    repo.upsert_by_external_id(CreateUser {
        external_id: "ext-list1".into(),
        email: "list1@example.com".into(),
        display_name: "User 1".into(),
    })
    .await
    .unwrap();
    repo.upsert_by_external_id(CreateUser {
        external_id: "ext-list2".into(),
        email: "list2@example.com".into(),
        display_name: "User 2".into(),
    })
    .await
    .unwrap();

    let all = repo.list_all().await.unwrap();
    assert!(all.len() >= 2);
}

#[tokio::test]
async fn user_delete() {
    let pool = test_db().await;
    let repo = user_repo::SqliteUserRepository::new(pool);

    let user = repo
        .upsert_by_external_id(CreateUser {
            external_id: "ext-del".into(),
            email: "del@example.com".into(),
            display_name: "Delete Me".into(),
        })
        .await
        .unwrap();

    repo.delete(user.id).await.unwrap();
    assert!(repo.get(user.id).await.unwrap().is_none());
}

#[tokio::test]
async fn user_count_admins() {
    let pool = test_db().await;
    let repo = user_repo::SqliteUserRepository::new(pool);

    assert_eq!(repo.count_admins().await.unwrap(), 0);

    let user = repo
        .upsert_by_external_id(CreateUser {
            external_id: "ext-ca".into(),
            email: "ca@example.com".into(),
            display_name: "CA".into(),
        })
        .await
        .unwrap();

    repo.set_admin(user.id, true).await.unwrap();
    assert_eq!(repo.count_admins().await.unwrap(), 1);
}

#[tokio::test]
async fn user_insert_raw_and_get() {
    let pool = test_db().await;
    let repo = user_repo::SqliteUserRepository::new(pool);

    let now = chrono::Utc::now();
    let user = User {
        id: Uuid::new_v4(),
        external_id: "raw-ext".into(),
        email: "raw@example.com".into(),
        display_name: "Raw User".into(),
        is_admin: true,
        created_at: now,
        updated_at: now,
    };

    repo.insert_raw(&user, Some("rawhash")).await.unwrap();

    let fetched = repo.get(user.id).await.unwrap().unwrap();
    assert_eq!(fetched.display_name, "Raw User");
    assert!(fetched.is_admin);

    let hash = repo.get_password_hash("raw-ext").await.unwrap().unwrap();
    assert_eq!(hash, "rawhash");
}

#[tokio::test]
async fn user_delete_all() {
    let pool = test_db().await;
    let repo = user_repo::SqliteUserRepository::new(pool);

    repo.upsert_by_external_id(CreateUser {
        external_id: "ext-da".into(),
        email: "da@example.com".into(),
        display_name: "DA".into(),
    })
    .await
    .unwrap();

    repo.delete_all().await.unwrap();

    let all = repo.list_all().await.unwrap();
    assert!(all.is_empty());
}

// =========================================================================
// Additional UserGroup Repository Tests
// =========================================================================

#[tokio::test]
async fn user_group_add_member() {
    let pool = test_db().await;
    let ug_repo = user_group_repo::SqliteUserGroupRepository::new(pool.clone());
    let g_repo = group_repo::SqliteGroupRepository::new(pool.clone());

    let user = create_test_user(&pool).await;
    let group = g_repo.get_or_create_by_name("add-test").await.unwrap();

    assert!(!ug_repo.is_member(user.id, group.id).await.unwrap());

    ug_repo
        .add_member(user.id, group.id, GroupRole::Member)
        .await
        .unwrap();

    assert!(ug_repo.is_member(user.id, group.id).await.unwrap());
}

#[tokio::test]
async fn user_group_remove_member() {
    let pool = test_db().await;
    let ug_repo = user_group_repo::SqliteUserGroupRepository::new(pool.clone());
    let g_repo = group_repo::SqliteGroupRepository::new(pool.clone());

    let user = create_test_user(&pool).await;
    let group = g_repo.get_or_create_by_name("remove-test").await.unwrap();

    ug_repo
        .add_member(user.id, group.id, GroupRole::Owner)
        .await
        .unwrap();
    assert!(ug_repo.is_member(user.id, group.id).await.unwrap());

    ug_repo.remove_member(user.id, group.id).await.unwrap();
    assert!(!ug_repo.is_member(user.id, group.id).await.unwrap());
}

#[tokio::test]
async fn user_group_list_members_of_group() {
    let pool = test_db().await;
    let ug_repo = user_group_repo::SqliteUserGroupRepository::new(pool.clone());
    let g_repo = group_repo::SqliteGroupRepository::new(pool.clone());

    let user1 = create_test_user(&pool).await;
    let user2 = create_test_user(&pool).await;
    let group = g_repo
        .get_or_create_by_name("list-members-test")
        .await
        .unwrap();

    ug_repo
        .add_member(user1.id, group.id, GroupRole::Owner)
        .await
        .unwrap();
    ug_repo
        .add_member(user2.id, group.id, GroupRole::Member)
        .await
        .unwrap();

    let members = ug_repo.list_members_of_group(group.id).await.unwrap();
    assert_eq!(members.len(), 2);

    let owner = members.iter().find(|(u, _)| u.id == user1.id).unwrap();
    assert_eq!(owner.1, GroupRole::Owner);
    let member = members.iter().find(|(u, _)| u.id == user2.id).unwrap();
    assert_eq!(member.1, GroupRole::Member);
}

#[tokio::test]
async fn user_group_list_all() {
    let pool = test_db().await;
    let ug_repo = user_group_repo::SqliteUserGroupRepository::new(pool.clone());
    let g_repo = group_repo::SqliteGroupRepository::new(pool.clone());

    let user = create_test_user(&pool).await;
    let group = g_repo.get_or_create_by_name("list-all-ug").await.unwrap();

    ug_repo
        .add_member(user.id, group.id, GroupRole::Member)
        .await
        .unwrap();

    let all = ug_repo.list_all().await.unwrap();
    assert!(!all.is_empty());
    assert!(
        all.iter()
            .any(|ug| ug.user_id == user.id && ug.group_id == group.id)
    );
}

#[tokio::test]
async fn user_group_insert_raw() {
    let pool = test_db().await;
    let ug_repo = user_group_repo::SqliteUserGroupRepository::new(pool.clone());
    let g_repo = group_repo::SqliteGroupRepository::new(pool.clone());

    let user = create_test_user(&pool).await;
    let group = g_repo.get_or_create_by_name("insert-raw-ug").await.unwrap();

    ug_repo
        .insert_raw(user.id, group.id, GroupRole::Owner)
        .await
        .unwrap();

    assert!(ug_repo.is_member(user.id, group.id).await.unwrap());

    let groups = ug_repo.list_groups_for_user(user.id).await.unwrap();
    let (_, role) = groups.iter().find(|(g, _)| g.id == group.id).unwrap();
    assert_eq!(*role, GroupRole::Owner);
}

#[tokio::test]
async fn user_group_delete_all() {
    let pool = test_db().await;
    let ug_repo = user_group_repo::SqliteUserGroupRepository::new(pool.clone());
    let g_repo = group_repo::SqliteGroupRepository::new(pool.clone());

    let user = create_test_user(&pool).await;
    let group = g_repo.get_or_create_by_name("delete-all-ug").await.unwrap();

    ug_repo
        .add_member(user.id, group.id, GroupRole::Member)
        .await
        .unwrap();

    ug_repo.delete_all().await.unwrap();

    let all = ug_repo.list_all().await.unwrap();
    assert!(all.is_empty());
}

// =========================================================================
// Additional Session Repository Tests
// =========================================================================

#[tokio::test]
async fn session_delete_all() {
    let pool = test_db().await;
    let s_repo = session_repo::SqliteSessionRepository::new(pool.clone());
    let user = create_test_user(&pool).await;

    s_repo
        .create(Session {
            id: "sess-da-1".into(),
            user_id: user.id,
            active_group_id: group_id(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
            created_at: chrono::Utc::now(),
        })
        .await
        .unwrap();
    s_repo
        .create(Session {
            id: "sess-da-2".into(),
            user_id: user.id,
            active_group_id: group_id(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
            created_at: chrono::Utc::now(),
        })
        .await
        .unwrap();

    s_repo.delete_all().await.unwrap();

    assert!(s_repo.get("sess-da-1").await.unwrap().is_none());
    assert!(s_repo.get("sess-da-2").await.unwrap().is_none());
}

// =========================================================================
// Additional Group Repository Tests
// =========================================================================

#[tokio::test]
async fn group_create_and_list_all() {
    let pool = test_db().await;
    let repo = group_repo::SqliteGroupRepository::new(pool);

    repo.create("test-group-1").await.unwrap();
    repo.create("test-group-2").await.unwrap();

    let all = repo.list_all().await.unwrap();
    // At least the 2 we created plus the seeded "default" group
    assert!(all.len() >= 3);
    assert!(all.iter().any(|g| g.name == "test-group-1"));
    assert!(all.iter().any(|g| g.name == "test-group-2"));
}

#[tokio::test]
async fn group_delete() {
    let pool = test_db().await;
    let repo = group_repo::SqliteGroupRepository::new(pool);

    let group = repo.create("deletable-group").await.unwrap();
    assert!(repo.get(group.id).await.unwrap().is_some());

    repo.delete(group.id).await.unwrap();
    assert!(repo.get(group.id).await.unwrap().is_none());
}

#[tokio::test]
async fn group_insert_raw_and_get() {
    let pool = test_db().await;
    let repo = group_repo::SqliteGroupRepository::new(pool);

    let now = chrono::Utc::now();
    let group = Group {
        id: Uuid::new_v4(),
        name: "raw-group".into(),
        created_at: now,
        updated_at: now,
    };

    repo.insert_raw(&group).await.unwrap();

    let fetched = repo.get(group.id).await.unwrap().unwrap();
    assert_eq!(fetched.name, "raw-group");
}

#[tokio::test]
async fn group_delete_all() {
    let pool = test_db().await;
    let repo = group_repo::SqliteGroupRepository::new(pool);

    repo.create("group-to-delete").await.unwrap();

    repo.delete_all().await.unwrap();

    let all = repo.list_all().await.unwrap();
    assert!(all.is_empty());
}
