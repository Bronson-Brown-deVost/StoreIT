use serde_json::{Value, json};

use crate::common::TestApp;

async fn setup_location(app: &TestApp) -> String {
    let res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "Room" }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    body["id"].as_str().unwrap().to_string()
}

async fn setup_container(app: &TestApp, loc_id: &str) -> String {
    let res = app
        .client
        .post(app.url("/api/v1/containers"))
        .json(&json!({ "parent_type": "location", "parent_id": loc_id, "name": "Box" }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    body["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn create_and_get_item() {
    let app = TestApp::spawn().await;
    let loc_id = setup_location(&app).await;
    let cont_id = setup_container(&app, &loc_id).await;

    let res = app
        .client
        .post(app.url("/api/v1/items"))
        .json(&json!({
            "parent_type": "container",
            "parent_id": cont_id,
            "name": "Screwdriver",
            "description": "Phillips head",
            "category": "Tools",
            "quantity": 3
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Screwdriver");
    assert_eq!(body["quantity"], 3);
    assert_eq!(body["category"], "Tools");
    let id = body["id"].as_str().unwrap();

    let res = app
        .client
        .get(app.url(&format!("/api/v1/items/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Screwdriver");
}

#[tokio::test]
async fn update_item() {
    let app = TestApp::spawn().await;
    let loc_id = setup_location(&app).await;
    let cont_id = setup_container(&app, &loc_id).await;

    let res = app
        .client
        .post(app.url("/api/v1/items"))
        .json(&json!({ "parent_type": "container", "parent_id": cont_id, "name": "Old" }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    let res = app
        .client
        .put(app.url(&format!("/api/v1/items/{id}")))
        .json(&json!({
            "name": "Updated",
            "aliases": ["alias1"],
            "keywords": ["kw1", "kw2"],
            "quantity": 5
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["name"], "Updated");
    assert_eq!(body["quantity"], 5);
    assert_eq!(body["aliases"], json!(["alias1"]));
}

#[tokio::test]
async fn delete_item() {
    let app = TestApp::spawn().await;
    let loc_id = setup_location(&app).await;
    let cont_id = setup_container(&app, &loc_id).await;

    let res = app
        .client
        .post(app.url("/api/v1/items"))
        .json(&json!({ "parent_type": "container", "parent_id": cont_id, "name": "Gone" }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    let res = app
        .client
        .delete(app.url(&format!("/api/v1/items/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    let res = app
        .client
        .get(app.url(&format!("/api/v1/items/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn move_item() {
    let app = TestApp::spawn().await;
    let loc_id = setup_location(&app).await;
    let cont_id = setup_container(&app, &loc_id).await;

    let res = app
        .client
        .post(app.url("/api/v1/items"))
        .json(&json!({ "parent_type": "container", "parent_id": cont_id, "name": "Mobile" }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();
    assert_eq!(body["container_id"], cont_id.as_str());

    let res = app
        .client
        .post(app.url(&format!("/api/v1/items/{id}/move")))
        .json(&json!({ "target_type": "location", "target_id": loc_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["location_id"], loc_id.as_str());
    assert!(body["container_id"].is_null());
}

#[tokio::test]
async fn batch_create_items() {
    let app = TestApp::spawn().await;
    let loc_id = setup_location(&app).await;
    let cont_id = setup_container(&app, &loc_id).await;

    let items = json!([
        { "parent_type": "container", "parent_id": cont_id, "name": "Item1" },
        { "parent_type": "container", "parent_id": cont_id, "name": "Item2" },
        { "parent_type": "container", "parent_id": cont_id, "name": "Item3" }
    ]);

    let res = app
        .client
        .post(app.url("/api/v1/items/batch"))
        .json(&items)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: Vec<Value> = res.json().await.unwrap();
    assert_eq!(body.len(), 3);
}

#[tokio::test]
async fn create_item_at_location() {
    let app = TestApp::spawn().await;
    let loc_id = setup_location(&app).await;

    let res = app
        .client
        .post(app.url("/api/v1/items"))
        .json(&json!({ "parent_type": "location", "parent_id": loc_id, "name": "DirectItem" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["location_id"], loc_id.as_str());
    assert!(body["container_id"].is_null());
}

#[tokio::test]
async fn create_item_invalid_parent_type() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/items"))
        .json(&json!({
            "parent_type": "invalid",
            "parent_id": "00000000-0000-0000-0000-000000000001",
            "name": "Nope"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn move_item_invalid_target_type() {
    let app = TestApp::spawn().await;
    let loc_id = setup_location(&app).await;
    let cont_id = setup_container(&app, &loc_id).await;

    let res = app
        .client
        .post(app.url("/api/v1/items"))
        .json(&json!({ "parent_type": "container", "parent_id": cont_id, "name": "MoveMe" }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    let res = app
        .client
        .post(app.url(&format!("/api/v1/items/{id}/move")))
        .json(&json!({ "target_type": "invalid", "target_id": loc_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn list_all_items() {
    let app = TestApp::spawn().await;
    let loc_id = setup_location(&app).await;
    let cont_id = setup_container(&app, &loc_id).await;

    for name in ["Item1", "Item2", "Item3"] {
        app.client
            .post(app.url("/api/v1/items"))
            .json(&json!({ "parent_type": "container", "parent_id": cont_id, "name": name }))
            .send()
            .await
            .unwrap();
    }

    let res = app
        .client
        .get(app.url("/api/v1/items"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Vec<Value> = res.json().await.unwrap();
    assert_eq!(body.len(), 3);
}
