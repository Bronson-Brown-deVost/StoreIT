use serde_json::{Value, json};

use crate::common::TestApp;

#[tokio::test]
async fn search_finds_created_entities() {
    let app = TestApp::spawn().await;

    // Create a location
    app.client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "Electronics Workshop" }))
        .send()
        .await
        .unwrap();

    // Create a container
    let loc_res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "Utility Room" }))
        .send()
        .await
        .unwrap();
    let loc: Value = loc_res.json().await.unwrap();
    let loc_id = loc["id"].as_str().unwrap();

    app.client
        .post(app.url("/api/v1/containers"))
        .json(&json!({
            "parent_type": "location",
            "parent_id": loc_id,
            "name": "Electronics Bin"
        }))
        .send()
        .await
        .unwrap();

    // Search for "electronics"
    let res = app
        .client
        .get(app.url("/api/v1/search?q=electronics&limit=10"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    assert!(results.len() >= 2, "expected at least 2 results");
}

#[tokio::test]
async fn search_empty_query_returns_results() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/v1/search?q=xyznonexistent"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    assert_eq!(results.len(), 0);
}
