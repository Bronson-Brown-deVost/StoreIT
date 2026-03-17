use crate::common::TestApp;
use reqwest::multipart;
use serde_json::Value;

#[tokio::test]
async fn identify_returns_200_with_valid_photo() {
    let app = TestApp::spawn().await;

    let form = multipart::Form::new().part(
        "photo",
        multipart::Part::bytes(b"fake-image-data".to_vec())
            .file_name("test.jpg")
            .mime_str("image/jpeg")
            .unwrap(),
    );

    let resp = app
        .client
        .post(app.url("/api/v1/identify"))
        .multipart(form)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["name"], "Red Stapler");
    assert_eq!(body["category"], "office supplies");
    assert_eq!(body["color"], "red");
    assert!(body["aliases"].is_array());
    assert!(body["keywords"].is_array());
}

#[tokio::test]
async fn identify_correct_returns_200() {
    let app = TestApp::spawn().await;

    let form = multipart::Form::new()
        .part(
            "photo",
            multipart::Part::bytes(b"fake-image-data".to_vec())
                .file_name("test.jpg")
                .mime_str("image/jpeg")
                .unwrap(),
        )
        .text("correction", "This is actually a blue pen");

    let resp = app
        .client
        .post(app.url("/api/v1/identify/correct"))
        .multipart(form)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["name"], "Corrected: This is actually a blue pen");
}

#[tokio::test]
async fn identify_without_photo_returns_400() {
    let app = TestApp::spawn().await;

    let form = multipart::Form::new().text("not_a_photo", "hello");

    let resp = app
        .client
        .post(app.url("/api/v1/identify"))
        .multipart(form)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn identify_correct_without_correction_returns_400() {
    let app = TestApp::spawn().await;

    let form = multipart::Form::new().part(
        "photo",
        multipart::Part::bytes(b"fake-image-data".to_vec())
            .file_name("test.jpg")
            .mime_str("image/jpeg")
            .unwrap(),
    );

    let resp = app
        .client
        .post(app.url("/api/v1/identify/correct"))
        .multipart(form)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
}
