use reqwest::multipart;
use serde_json::{Value, json};

use crate::common::TestApp;

async fn create_location(app: &TestApp) -> String {
    let res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&json!({ "name": "PhotoRoom" }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    body["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn upload_and_get_photo() {
    let app = TestApp::spawn().await;
    let loc_id = create_location(&app).await;

    let form = multipart::Form::new()
        .text("entity_type", "location")
        .text("entity_id", loc_id.clone())
        .part(
            "file",
            multipart::Part::bytes(b"fake-image-data".to_vec())
                .file_name("test.png")
                .mime_str("image/png")
                .unwrap(),
        );

    let res = app
        .client
        .post(app.url("/api/v1/photos"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["entity_type"], "location");
    assert_eq!(body["entity_id"], loc_id.as_str());
    assert_eq!(body["mime_type"], "image/png");
    let photo_id = body["id"].as_str().unwrap();

    // Get metadata
    let res = app
        .client
        .get(app.url(&format!("/api/v1/photos/{photo_id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["id"], photo_id);

    // Get file
    let res = app
        .client
        .get(app.url(&format!("/api/v1/photos/{photo_id}/file")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let data = res.bytes().await.unwrap();
    assert_eq!(&data[..], b"fake-image-data");
}

#[tokio::test]
async fn list_entity_photos() {
    let app = TestApp::spawn().await;
    let loc_id = create_location(&app).await;

    // Upload two photos
    for i in 0..2 {
        let form = multipart::Form::new()
            .text("entity_type", "location")
            .text("entity_id", loc_id.clone())
            .part(
                "file",
                multipart::Part::bytes(format!("data{i}").into_bytes())
                    .file_name("test.jpg")
                    .mime_str("image/jpeg")
                    .unwrap(),
            );
        app.client
            .post(app.url("/api/v1/photos"))
            .multipart(form)
            .send()
            .await
            .unwrap();
    }

    let res = app
        .client
        .get(app.url(&format!(
            "/api/v1/photos/by-entity?entity_type=location&entity_id={loc_id}"
        )))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let photos: Vec<Value> = res.json().await.unwrap();
    assert_eq!(photos.len(), 2);
}

#[tokio::test]
async fn delete_photo() {
    let app = TestApp::spawn().await;
    let loc_id = create_location(&app).await;

    let form = multipart::Form::new()
        .text("entity_type", "location")
        .text("entity_id", loc_id.clone())
        .part(
            "file",
            multipart::Part::bytes(b"delete-me".to_vec())
                .file_name("rm.png")
                .mime_str("image/png")
                .unwrap(),
        );

    let res = app
        .client
        .post(app.url("/api/v1/photos"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    let photo_id = body["id"].as_str().unwrap();

    let res = app
        .client
        .delete(app.url(&format!("/api/v1/photos/{photo_id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    let res = app
        .client
        .get(app.url(&format!("/api/v1/photos/{photo_id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn upload_missing_fields() {
    let app = TestApp::spawn().await;

    // Missing entity_type
    let form = multipart::Form::new()
        .text("entity_id", "00000000-0000-0000-0000-000000000001")
        .part(
            "file",
            multipart::Part::bytes(b"data".to_vec())
                .file_name("x.png")
                .mime_str("image/png")
                .unwrap(),
        );

    let res = app
        .client
        .post(app.url("/api/v1/photos"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn upload_invalid_entity_type() {
    let app = TestApp::spawn().await;

    let form = multipart::Form::new()
        .text("entity_type", "invalid")
        .text("entity_id", "00000000-0000-0000-0000-000000000001")
        .part(
            "file",
            multipart::Part::bytes(b"data".to_vec())
                .file_name("x.png")
                .mime_str("image/png")
                .unwrap(),
        );

    let res = app
        .client
        .post(app.url("/api/v1/photos"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn upload_invalid_entity_id() {
    let app = TestApp::spawn().await;

    let form = multipart::Form::new()
        .text("entity_type", "location")
        .text("entity_id", "not-a-uuid")
        .part(
            "file",
            multipart::Part::bytes(b"data".to_vec())
                .file_name("x.png")
                .mime_str("image/png")
                .unwrap(),
        );

    let res = app
        .client
        .post(app.url("/api/v1/photos"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn upload_missing_file() {
    let app = TestApp::spawn().await;

    let form = multipart::Form::new()
        .text("entity_type", "location")
        .text("entity_id", "00000000-0000-0000-0000-000000000001");

    let res = app
        .client
        .post(app.url("/api/v1/photos"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn upload_missing_entity_id() {
    let app = TestApp::spawn().await;

    let form = multipart::Form::new().text("entity_type", "location").part(
        "file",
        multipart::Part::bytes(b"data".to_vec())
            .file_name("x.png")
            .mime_str("image/png")
            .unwrap(),
    );

    let res = app
        .client
        .post(app.url("/api/v1/photos"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn list_photos_invalid_entity_type() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .get(app.url(
            "/api/v1/photos/by-entity?entity_type=invalid&entity_id=00000000-0000-0000-0000-000000000001",
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn get_photo_not_found() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/v1/photos/00000000-0000-0000-0000-000000000099"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn get_photo_file_not_found() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/v1/photos/00000000-0000-0000-0000-000000000099/file"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn delete_photo_not_found() {
    let app = TestApp::spawn().await;

    let res = app
        .client
        .delete(app.url("/api/v1/photos/00000000-0000-0000-0000-000000000099"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn upload_various_mime_types() {
    let app = TestApp::spawn().await;
    let loc_id = create_location(&app).await;

    // Upload gif
    let form = multipart::Form::new()
        .text("entity_type", "location")
        .text("entity_id", loc_id.clone())
        .part(
            "file",
            multipart::Part::bytes(b"gif-data".to_vec())
                .file_name("test.gif")
                .mime_str("image/gif")
                .unwrap(),
        );
    let res = app
        .client
        .post(app.url("/api/v1/photos"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["mime_type"], "image/gif");

    // Upload webp
    let form = multipart::Form::new()
        .text("entity_type", "location")
        .text("entity_id", loc_id.clone())
        .part(
            "file",
            multipart::Part::bytes(b"webp-data".to_vec())
                .file_name("test.webp")
                .mime_str("image/webp")
                .unwrap(),
        );
    let res = app
        .client
        .post(app.url("/api/v1/photos"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    // Upload unknown mime type (triggers fallback extension)
    let form = multipart::Form::new()
        .text("entity_type", "location")
        .text("entity_id", loc_id.clone())
        .part(
            "file",
            multipart::Part::bytes(b"raw-data".to_vec())
                .file_name("test.bin")
                .mime_str("application/octet-stream")
                .unwrap(),
        );
    let res = app
        .client
        .post(app.url("/api/v1/photos"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
}

#[tokio::test]
async fn upload_with_extra_fields_ignored() {
    let app = TestApp::spawn().await;
    let loc_id = create_location(&app).await;

    let form = multipart::Form::new()
        .text("entity_type", "location")
        .text("entity_id", loc_id)
        .text("extra_field", "should be ignored")
        .part(
            "file",
            multipart::Part::bytes(b"data".to_vec())
                .file_name("test.png")
                .mime_str("image/png")
                .unwrap(),
        );
    let res = app
        .client
        .post(app.url("/api/v1/photos"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
}
