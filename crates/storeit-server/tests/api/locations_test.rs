use serde_json::{Value, json};

use crate::common::TestApp;

#[tokio::test]
async fn create_and_get_location() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "Garage", "description": "Main garage" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Garage");
    assert_eq!(body["description"], "Main garage");
    let id = body["id"].as_str().unwrap();

    let res = app
        .client
        .get(app.url(&format!("/api/v1/locations/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Garage");
}

#[tokio::test]
async fn list_roots() {
    let app = TestApp::spawn().await;

    app.client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "Basement" }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "Attic" }))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .get(app.url("/api/v1/locations"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Vec<Value> = res.json().await.unwrap();
    assert_eq!(body.len(), 2);
}

#[tokio::test]
async fn update_location() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "Old Name" }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    let res = app
        .client
        .put(app.url(&format!("/api/v1/locations/{id}")))
        .json(&json!({ "name": "New Name", "description": "Updated" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["name"], "New Name");
    assert_eq!(body["description"], "Updated");
}

#[tokio::test]
async fn delete_location() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "To Delete" }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    let res = app
        .client
        .delete(app.url(&format!("/api/v1/locations/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    let res = app
        .client
        .get(app.url(&format!("/api/v1/locations/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn delete_non_empty_location_fails() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "Parent" }))
        .send()
        .await
        .unwrap();
    let parent: Value = res.json().await.unwrap();
    let parent_id = parent["id"].as_str().unwrap();

    // Create child location
    app.client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "Child", "parent_id": parent_id }))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .delete(app.url(&format!("/api/v1/locations/{parent_id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);
}

#[tokio::test]
async fn get_tree() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "Root" }))
        .send()
        .await
        .unwrap();
    let root: Value = res.json().await.unwrap();
    let root_id = root["id"].as_str().unwrap();

    app.client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "Child", "parent_id": root_id }))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .get(app.url("/api/v1/locations/tree"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let tree: Vec<Value> = res.json().await.unwrap();
    assert_eq!(tree.len(), 1);
    assert_eq!(tree[0]["name"], "Root");
    assert_eq!(tree[0]["children"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn list_children() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "Parent" }))
        .send()
        .await
        .unwrap();
    let parent: Value = res.json().await.unwrap();
    let parent_id = parent["id"].as_str().unwrap();

    app.client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "Kid1", "parent_id": parent_id }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "Kid2", "parent_id": parent_id }))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .get(app.url(&format!("/api/v1/locations/{parent_id}/children")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let children: Vec<Value> = res.json().await.unwrap();
    assert_eq!(children.len(), 2);
}

#[tokio::test]
async fn get_not_found() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/v1/locations/00000000-0000-0000-0000-000000000099"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn create_location_with_coordinates() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&json!({
            "name": "GPS Place",
            "latitude": 45.5231,
            "longitude": -122.6765
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["name"], "GPS Place");
    assert!((body["latitude"].as_f64().unwrap() - 45.5231).abs() < 1e-4);
    assert!((body["longitude"].as_f64().unwrap() - (-122.6765)).abs() < 1e-4);

    // Verify via GET
    let id = body["id"].as_str().unwrap();
    let res = app
        .client
        .get(app.url(&format!("/api/v1/locations/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert!((body["latitude"].as_f64().unwrap() - 45.5231).abs() < 1e-4);
    assert!((body["longitude"].as_f64().unwrap() - (-122.6765)).abs() < 1e-4);
}

#[tokio::test]
async fn create_location_without_coordinates_returns_null() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "No GPS" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    assert!(body["latitude"].is_null());
    assert!(body["longitude"].is_null());
}

#[tokio::test]
async fn update_location_coordinates() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "Updatable" }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();
    assert!(body["latitude"].is_null());

    // Update with coordinates
    let res = app
        .client
        .put(app.url(&format!("/api/v1/locations/{id}")))
        .json(&json!({ "latitude": 34.0522, "longitude": -118.2437 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert!((body["latitude"].as_f64().unwrap() - 34.0522).abs() < 1e-4);
    assert!((body["longitude"].as_f64().unwrap() - (-118.2437)).abs() < 1e-4);
}

#[tokio::test]
async fn coordinates_in_tree_response() {
    let app = TestApp::spawn().await;

    app.client
        .post(app.url("/api/v1/locations"))
        .json(&json!({
            "name": "Tree Node",
            "latitude": 51.5074,
            "longitude": -0.1278
        }))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .get(app.url("/api/v1/locations/tree"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let tree: Vec<Value> = res.json().await.unwrap();
    assert_eq!(tree.len(), 1);
    assert!((tree[0]["latitude"].as_f64().unwrap() - 51.5074).abs() < 1e-4);
    assert!((tree[0]["longitude"].as_f64().unwrap() - (-0.1278)).abs() < 1e-4);
}

#[tokio::test]
async fn coordinates_in_children_response() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "Parent" }))
        .send()
        .await
        .unwrap();
    let parent: Value = res.json().await.unwrap();
    let parent_id = parent["id"].as_str().unwrap();

    app.client
        .post(app.url("/api/v1/locations"))
        .json(&json!({
            "name": "Child with GPS",
            "parent_id": parent_id,
            "latitude": 35.6762,
            "longitude": 139.6503
        }))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .get(app.url(&format!("/api/v1/locations/{parent_id}/children")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let children: Vec<Value> = res.json().await.unwrap();
    assert_eq!(children.len(), 1);
    assert!((children[0]["latitude"].as_f64().unwrap() - 35.6762).abs() < 1e-4);
    assert!((children[0]["longitude"].as_f64().unwrap() - 139.6503).abs() < 1e-4);
}

#[tokio::test]
async fn list_location_containers_and_items() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "Office" }))
        .send()
        .await
        .unwrap();
    let loc: Value = res.json().await.unwrap();
    let loc_id = loc["id"].as_str().unwrap();

    // Create container in location
    app.client
        .post(app.url("/api/v1/containers"))
        .json(&json!({ "parent_type": "location", "parent_id": loc_id, "name": "Shelf" }))
        .send()
        .await
        .unwrap();

    // Create item in location
    app.client
        .post(app.url("/api/v1/items"))
        .json(&json!({ "parent_type": "location", "parent_id": loc_id, "name": "Lamp" }))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .get(app.url(&format!("/api/v1/locations/{loc_id}/containers")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let containers: Vec<Value> = res.json().await.unwrap();
    assert_eq!(containers.len(), 1);

    let res = app
        .client
        .get(app.url(&format!("/api/v1/locations/{loc_id}/items")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let items: Vec<Value> = res.json().await.unwrap();
    assert_eq!(items.len(), 1);
}
