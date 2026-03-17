use serde_json::{Value, json};

use crate::common::TestApp;

async fn create_location(app: &TestApp, name: &str) -> String {
    let res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": name }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    body["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn create_and_get_container() {
    let app = TestApp::spawn().await;
    let loc_id = create_location(&app, "Room").await;

    let res = app
        .client
        .post(app.url("/api/v1/containers"))
        .json(&json!({
            "parent_type": "location",
            "parent_id": loc_id,
            "name": "Box A",
            "color": "#FF0000"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Box A");
    assert_eq!(body["color"], "#FF0000");
    let id = body["id"].as_str().unwrap();

    let res = app
        .client
        .get(app.url(&format!("/api/v1/containers/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Box A");
}

#[tokio::test]
async fn update_container() {
    let app = TestApp::spawn().await;
    let loc_id = create_location(&app, "Room").await;

    let res = app
        .client
        .post(app.url("/api/v1/containers"))
        .json(&json!({ "parent_type": "location", "parent_id": loc_id, "name": "Old" }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    let res = app
        .client
        .put(app.url(&format!("/api/v1/containers/{id}")))
        .json(&json!({ "name": "Renamed", "color": "#00FF00" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Renamed");
    assert_eq!(body["color"], "#00FF00");
}

#[tokio::test]
async fn delete_container() {
    let app = TestApp::spawn().await;
    let loc_id = create_location(&app, "Room").await;

    let res = app
        .client
        .post(app.url("/api/v1/containers"))
        .json(&json!({ "parent_type": "location", "parent_id": loc_id, "name": "Gone" }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    let res = app
        .client
        .delete(app.url(&format!("/api/v1/containers/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    let res = app
        .client
        .get(app.url(&format!("/api/v1/containers/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn delete_non_empty_container_fails() {
    let app = TestApp::spawn().await;
    let loc_id = create_location(&app, "Room").await;

    let res = app
        .client
        .post(app.url("/api/v1/containers"))
        .json(&json!({ "parent_type": "location", "parent_id": loc_id, "name": "Parent" }))
        .send()
        .await
        .unwrap();
    let parent: Value = res.json().await.unwrap();
    let parent_id = parent["id"].as_str().unwrap();

    // Create child container
    app.client
        .post(app.url("/api/v1/containers"))
        .json(&json!({ "parent_type": "container", "parent_id": parent_id, "name": "Child" }))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .delete(app.url(&format!("/api/v1/containers/{parent_id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);
}

#[tokio::test]
async fn move_container() {
    let app = TestApp::spawn().await;
    let loc1 = create_location(&app, "Room1").await;
    let loc2 = create_location(&app, "Room2").await;

    let res = app
        .client
        .post(app.url("/api/v1/containers"))
        .json(&json!({ "parent_type": "location", "parent_id": loc1, "name": "Mobile" }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();
    assert_eq!(body["parent_location_id"], loc1.as_str());

    let res = app
        .client
        .post(app.url(&format!("/api/v1/containers/{id}/move")))
        .json(&json!({ "target_type": "location", "target_id": loc2 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["parent_location_id"], loc2.as_str());
}

#[tokio::test]
async fn circular_move_rejected() {
    let app = TestApp::spawn().await;
    let loc_id = create_location(&app, "Room").await;

    let res = app
        .client
        .post(app.url("/api/v1/containers"))
        .json(&json!({ "parent_type": "location", "parent_id": loc_id, "name": "A" }))
        .send()
        .await
        .unwrap();
    let a: Value = res.json().await.unwrap();
    let a_id = a["id"].as_str().unwrap();

    let res = app
        .client
        .post(app.url("/api/v1/containers"))
        .json(&json!({ "parent_type": "container", "parent_id": a_id, "name": "B" }))
        .send()
        .await
        .unwrap();
    let b: Value = res.json().await.unwrap();
    let b_id = b["id"].as_str().unwrap();

    // Try to move A into B (its child) — should fail
    let res = app
        .client
        .post(app.url(&format!("/api/v1/containers/{a_id}/move")))
        .json(&json!({ "target_type": "container", "target_id": b_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);
}

#[tokio::test]
async fn get_ancestry() {
    let app = TestApp::spawn().await;
    let loc_id = create_location(&app, "Building").await;

    let res = app
        .client
        .post(app.url("/api/v1/containers"))
        .json(&json!({ "parent_type": "location", "parent_id": loc_id, "name": "Floor1" }))
        .send()
        .await
        .unwrap();
    let floor: Value = res.json().await.unwrap();
    let floor_id = floor["id"].as_str().unwrap();

    let res = app
        .client
        .post(app.url("/api/v1/containers"))
        .json(&json!({ "parent_type": "container", "parent_id": floor_id, "name": "Room101" }))
        .send()
        .await
        .unwrap();
    let room: Value = res.json().await.unwrap();
    let room_id = room["id"].as_str().unwrap();

    let res = app
        .client
        .get(app.url(&format!("/api/v1/containers/{room_id}/ancestry")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let ancestry: Vec<Value> = res.json().await.unwrap();
    assert!(ancestry.len() >= 2); // at least location + parent container + self
}

#[tokio::test]
async fn list_child_containers_and_items() {
    let app = TestApp::spawn().await;
    let loc_id = create_location(&app, "Room").await;

    let res = app
        .client
        .post(app.url("/api/v1/containers"))
        .json(&json!({ "parent_type": "location", "parent_id": loc_id, "name": "Parent" }))
        .send()
        .await
        .unwrap();
    let parent: Value = res.json().await.unwrap();
    let parent_id = parent["id"].as_str().unwrap();

    app.client
        .post(app.url("/api/v1/containers"))
        .json(&json!({ "parent_type": "container", "parent_id": parent_id, "name": "Sub" }))
        .send()
        .await
        .unwrap();

    app.client
        .post(app.url("/api/v1/items"))
        .json(&json!({ "parent_type": "container", "parent_id": parent_id, "name": "Widget" }))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .get(app.url(&format!("/api/v1/containers/{parent_id}/containers")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let subs: Vec<Value> = res.json().await.unwrap();
    assert_eq!(subs.len(), 1);

    let res = app
        .client
        .get(app.url(&format!("/api/v1/containers/{parent_id}/items")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let items: Vec<Value> = res.json().await.unwrap();
    assert_eq!(items.len(), 1);
}

#[tokio::test]
async fn create_container_invalid_parent_type() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/containers"))
        .json(&json!({
            "parent_type": "invalid",
            "parent_id": "00000000-0000-0000-0000-000000000001",
            "name": "Bad"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn move_container_invalid_target_type() {
    let app = TestApp::spawn().await;
    let loc_id = create_location(&app, "Room").await;

    let res = app
        .client
        .post(app.url("/api/v1/containers"))
        .json(&json!({ "parent_type": "location", "parent_id": loc_id, "name": "MoveBad" }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    let res = app
        .client
        .post(app.url(&format!("/api/v1/containers/{id}/move")))
        .json(&json!({ "target_type": "invalid", "target_id": loc_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn list_all_containers() {
    let app = TestApp::spawn().await;
    let loc_id = create_location(&app, "Room").await;

    for name in ["Box1", "Box2", "Box3"] {
        app.client
            .post(app.url("/api/v1/containers"))
            .json(&json!({ "parent_type": "location", "parent_id": loc_id, "name": name }))
            .send()
            .await
            .unwrap();
    }

    let res = app
        .client
        .get(app.url("/api/v1/containers"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Vec<Value> = res.json().await.unwrap();
    assert_eq!(body.len(), 3);
}
