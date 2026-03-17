use crate::common::{AdminTestApp, TestApp};

#[tokio::test]
async fn get_settings_requires_auth() {
    let app = TestApp::spawn().await;

    let bare = reqwest::Client::new();
    let res = bare
        .get(app.url("/api/v1/admin/settings"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 401);
}

#[tokio::test]
async fn get_settings_requires_admin() {
    let app = AdminTestApp::spawn().await;

    // Non-admin user should get 403
    let res = app
        .non_admin_client
        .get(app.url("/api/v1/admin/settings"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn get_settings_returns_current_path() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/v1/admin/settings"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body["image_storage_path"].is_string());
    assert_eq!(body["image_storage_path_readonly"], false);
}

#[tokio::test]
async fn update_settings_changes_image_path() {
    let app = AdminTestApp::spawn().await;

    let new_dir = tempfile::TempDir::new().unwrap();
    let new_path = new_dir.path().join("new_images");
    let new_path_str = new_path.to_string_lossy().to_string();

    let res = app
        .client
        .put(app.url("/api/v1/admin/settings"))
        .json(&serde_json::json!({ "image_storage_path": new_path_str }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["image_storage_path"], new_path_str);
    assert_eq!(body["image_storage_path_readonly"], false);

    // Verify GET returns the new path
    let res = app
        .client
        .get(app.url("/api/v1/admin/settings"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["image_storage_path"], new_path_str);

    // Verify directory was created
    assert!(new_path.exists());
}

#[tokio::test]
async fn update_settings_readonly_returns_409() {
    let app = AdminTestApp::spawn_with_env_image_path(true).await;

    let res = app
        .client
        .put(app.url("/api/v1/admin/settings"))
        .json(&serde_json::json!({ "image_storage_path": "/new/path" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 409);
}

#[tokio::test]
async fn update_settings_persists_to_db() {
    let app = AdminTestApp::spawn().await;

    let new_dir = tempfile::TempDir::new().unwrap();
    let new_path = new_dir.path().join("persisted_images");
    let new_path_str = new_path.to_string_lossy().to_string();

    let res = app
        .client
        .put(app.url("/api/v1/admin/settings"))
        .json(&serde_json::json!({ "image_storage_path": new_path_str }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    // Verify the value was persisted in the settings repo
    let db_val = app.settings_repo.get("image_storage_path").await.unwrap();
    assert_eq!(db_val, Some(new_path_str));
}

#[tokio::test]
async fn update_settings_rejects_empty_path() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .client
        .put(app.url("/api/v1/admin/settings"))
        .json(&serde_json::json!({ "image_storage_path": "" }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
}
