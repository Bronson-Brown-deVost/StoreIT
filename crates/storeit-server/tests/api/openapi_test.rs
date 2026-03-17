use serde_json::Value;

use crate::common::TestApp;

#[tokio::test]
async fn openapi_spec_is_valid_json() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api-docs/openapi.json"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let spec: Value = res.json().await.unwrap();
    assert_eq!(spec["info"]["title"], "StoreIT API");
    assert_eq!(spec["info"]["version"], "0.1.0");
    assert!(spec["paths"].is_object());
}

#[tokio::test]
async fn swagger_ui_accessible() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/swagger-ui/"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
}
