use serde_json::{Value, json};
use uuid::Uuid;

use crate::common::TestApp;

fn percent_encode(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

async fn setup_location(app: &TestApp) -> String {
    let res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "NFC Room" }))
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
        .json(&json!({ "parent_type": "location", "parent_id": loc_id, "name": "NFC Box" }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    body["id"].as_str().unwrap().to_string()
}

async fn create_tag(app: &TestApp, uri: &str) -> Value {
    let res = app
        .client
        .post(app.url("/api/v1/nfc-tags"))
        .json(&json!({ "tag_uri": uri }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    res.json().await.unwrap()
}

#[tokio::test]
async fn create_nfc_tag() {
    let app = TestApp::spawn().await;

    let body = create_tag(&app, "nfc://tag/001").await;
    assert!(body["id"].as_str().is_some());
    assert_eq!(body["tag_uri"], "nfc://tag/001");
    assert!(body["entity_type"].is_null());
    assert!(body["entity_id"].is_null());
}

#[tokio::test]
async fn list_nfc_tags() {
    let app = TestApp::spawn().await;

    create_tag(&app, "nfc://tag/list-1").await;
    create_tag(&app, "nfc://tag/list-2").await;

    let res = app
        .client
        .get(app.url("/api/v1/nfc-tags"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Vec<Value> = res.json().await.unwrap();
    assert_eq!(body.len(), 2);
}

#[tokio::test]
async fn get_nfc_tag() {
    let app = TestApp::spawn().await;

    let tag = create_tag(&app, "nfc://tag/get-1").await;
    let id = tag["id"].as_str().unwrap();

    let res = app
        .client
        .get(app.url(&format!("/api/v1/nfc-tags/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["tag_uri"], "nfc://tag/get-1");
}

#[tokio::test]
async fn get_nfc_tag_not_found() {
    let app = TestApp::spawn().await;
    let random_id = Uuid::new_v4();

    let res = app
        .client
        .get(app.url(&format!("/api/v1/nfc-tags/{random_id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn assign_to_container() {
    let app = TestApp::spawn().await;
    let loc_id = setup_location(&app).await;
    let cont_id = setup_container(&app, &loc_id).await;

    let tag = create_tag(&app, "nfc://tag/assign-container").await;
    let id = tag["id"].as_str().unwrap();

    let res = app
        .client
        .put(app.url(&format!("/api/v1/nfc-tags/{id}/assign")))
        .json(&json!({ "entity_type": "container", "entity_id": cont_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["entity_type"], "container");
    assert_eq!(body["entity_id"], cont_id.as_str());
}

#[tokio::test]
async fn assign_to_location() {
    let app = TestApp::spawn().await;
    let loc_id = setup_location(&app).await;

    let tag = create_tag(&app, "nfc://tag/assign-location").await;
    let id = tag["id"].as_str().unwrap();

    let res = app
        .client
        .put(app.url(&format!("/api/v1/nfc-tags/{id}/assign")))
        .json(&json!({ "entity_type": "location", "entity_id": loc_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["entity_type"], "location");
    assert_eq!(body["entity_id"], loc_id.as_str());
}

#[tokio::test]
async fn assign_to_item_rejected() {
    let app = TestApp::spawn().await;

    let tag = create_tag(&app, "nfc://tag/assign-item").await;
    let id = tag["id"].as_str().unwrap();

    let res = app
        .client
        .put(app.url(&format!("/api/v1/nfc-tags/{id}/assign")))
        .json(&json!({
            "entity_type": "item",
            "entity_id": "00000000-0000-0000-0000-000000000001"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn unassign_nfc_tag() {
    let app = TestApp::spawn().await;
    let loc_id = setup_location(&app).await;

    let tag = create_tag(&app, "nfc://tag/unassign").await;
    let id = tag["id"].as_str().unwrap();

    // Assign first
    app.client
        .put(app.url(&format!("/api/v1/nfc-tags/{id}/assign")))
        .json(&json!({ "entity_type": "location", "entity_id": loc_id }))
        .send()
        .await
        .unwrap();

    // Then unassign
    let res = app
        .client
        .put(app.url(&format!("/api/v1/nfc-tags/{id}/unassign")))
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);
}

#[tokio::test]
async fn delete_nfc_tag() {
    let app = TestApp::spawn().await;

    let tag = create_tag(&app, "nfc://tag/delete").await;
    let id = tag["id"].as_str().unwrap();

    let res = app
        .client
        .delete(app.url(&format!("/api/v1/nfc-tags/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    // Verify it's gone
    let res = app
        .client
        .get(app.url(&format!("/api/v1/nfc-tags/{id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn resolve_nfc_tag() {
    let app = TestApp::spawn().await;
    let loc_id = setup_location(&app).await;
    let cont_id = setup_container(&app, &loc_id).await;

    let tag = create_tag(&app, "nfc://tag/resolve").await;
    let id = tag["id"].as_str().unwrap();

    // Assign to container
    app.client
        .put(app.url(&format!("/api/v1/nfc-tags/{id}/assign")))
        .json(&json!({ "entity_type": "container", "entity_id": cont_id }))
        .send()
        .await
        .unwrap();

    let encoded_uri = percent_encode("nfc://tag/resolve");
    let res = app
        .client
        .get(app.url(&format!("/api/v1/nfc-tags/resolve/{encoded_uri}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["entity_type"], "container");
    assert_eq!(body["entity_name"], "NFC Box");
    assert!(body["location_path"].is_array());
}

#[tokio::test]
async fn resolve_uid_unknown() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/v1/nfc-tags/resolve-uid?uid=04A3B2C1D5E6F7"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["status"], "unknown");
    assert!(body.get("tag_id").is_none() || body["tag_id"].is_null());
}

#[tokio::test]
async fn resolve_uid_unassigned() {
    let app = TestApp::spawn().await;

    create_tag(&app, "04AABB11223344").await;

    let res = app
        .client
        .get(app.url("/api/v1/nfc-tags/resolve-uid?uid=04AABB11223344"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["status"], "unassigned");
    assert!(body["tag_id"].as_str().is_some());
}

#[tokio::test]
async fn resolve_uid_assigned() {
    let app = TestApp::spawn().await;
    let loc_id = setup_location(&app).await;

    let tag = create_tag(&app, "04CCDD55667788").await;
    let id = tag["id"].as_str().unwrap();

    app.client
        .put(app.url(&format!("/api/v1/nfc-tags/{id}/assign")))
        .json(&json!({ "entity_type": "location", "entity_id": loc_id }))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .get(app.url("/api/v1/nfc-tags/resolve-uid?uid=04CCDD55667788"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["status"], "assigned");
    assert_eq!(body["entity_type"], "location");
    assert_eq!(body["entity_id"], loc_id.as_str());
    assert_eq!(body["entity_name"], "NFC Room");
}

#[tokio::test]
async fn register_and_assign() {
    let app = TestApp::spawn().await;
    let loc_id = setup_location(&app).await;
    let cont_id = setup_container(&app, &loc_id).await;

    let res = app
        .client
        .post(app.url("/api/v1/nfc-tags/register-and-assign"))
        .json(&json!({
            "tag_uri": "04EEFF99887766",
            "entity_type": "container",
            "entity_id": cont_id,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["tag_uri"], "04EEFF99887766");
    assert_eq!(body["entity_type"], "container");
    assert_eq!(body["entity_id"], cont_id.as_str());
}

#[tokio::test]
async fn register_and_assign_existing_tag() {
    let app = TestApp::spawn().await;
    let loc_id = setup_location(&app).await;

    // Pre-create the tag (unassigned)
    create_tag(&app, "04112233445566").await;

    // register-and-assign should find it and assign
    let res = app
        .client
        .post(app.url("/api/v1/nfc-tags/register-and-assign"))
        .json(&json!({
            "tag_uri": "04112233445566",
            "entity_type": "location",
            "entity_id": loc_id,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["entity_type"], "location");
    assert_eq!(body["entity_id"], loc_id.as_str());
}

#[tokio::test]
async fn register_and_assign_rejects_item() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .post(app.url("/api/v1/nfc-tags/register-and-assign"))
        .json(&json!({
            "tag_uri": "04AABBCCDDEEFF",
            "entity_type": "item",
            "entity_id": "00000000-0000-0000-0000-000000000001",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}
