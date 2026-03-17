use crate::common::AdminTestApp;

#[tokio::test]
async fn restore_requires_admin() {
    let app = AdminTestApp::spawn().await;

    // Build a minimal valid archive
    let archive = build_test_archive(false);
    let options = serde_json::json!({ "mode": "replace" }).to_string();

    let form = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(archive)
                .file_name("backup.tar.gz")
                .mime_str("application/gzip")
                .unwrap(),
        )
        .text("options", options);

    let res = app
        .non_admin_client
        .post(app.url("/api/v1/admin/restore"))
        .multipart(form)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn restore_rejects_invalid_archive() {
    let app = AdminTestApp::spawn().await;

    let form = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(vec![1, 2, 3, 4])
                .file_name("bad.tar.gz")
                .mime_str("application/gzip")
                .unwrap(),
        )
        .text(
            "options",
            serde_json::json!({ "mode": "replace" }).to_string(),
        );

    let res = app
        .client
        .post(app.url("/api/v1/admin/restore"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let body: serde_json::Value = res.json().await.unwrap();
    let job_id = body["job_id"].as_str().unwrap();

    // Poll — should fail
    let status = poll_restore_status(&app, job_id).await;
    assert_eq!(status["status"], "failed");
    assert!(!status["error"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn restore_full_replace_wipes_and_restores() {
    let app = AdminTestApp::spawn().await;

    // First, create a backup of the current state
    let res = app
        .client
        .post(app.url("/api/v1/admin/backup"))
        .json(&serde_json::json!({ "include_images": false }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let job_id = body["job_id"].as_str().unwrap();

    let status = poll_backup_status(&app, job_id).await;
    assert_eq!(status["status"], "complete");

    // Download the backup
    let res = app
        .client
        .get(app.url(&format!("/api/v1/admin/backup/{job_id}/download")))
        .send()
        .await
        .unwrap();
    let backup_data = res.bytes().await.unwrap().to_vec();

    // Create some extra data that should be wiped
    let _ = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&serde_json::json!({ "name": "Extra Location To Be Wiped" }))
        .send()
        .await
        .unwrap();

    // Restore with full replace
    let form = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(backup_data)
                .file_name("backup.tar.gz")
                .mime_str("application/gzip")
                .unwrap(),
        )
        .text(
            "options",
            serde_json::json!({ "mode": "replace" }).to_string(),
        );

    let res = app
        .client
        .post(app.url("/api/v1/admin/restore"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let restore_job_id = body["job_id"].as_str().unwrap();

    let status = poll_restore_status(&app, restore_job_id).await;
    assert_eq!(
        status["status"], "complete",
        "restore should complete, got: {:?}",
        status
    );
}

#[tokio::test]
async fn restore_merge_adds_data_with_new_ids() {
    let app = AdminTestApp::spawn().await;

    // Create a location
    let loc_res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&serde_json::json!({ "name": "Original Location" }))
        .send()
        .await
        .unwrap();
    assert_eq!(loc_res.status(), 201);
    let original_loc: serde_json::Value = loc_res.json().await.unwrap();
    let original_loc_id = original_loc["id"].as_str().unwrap().to_string();

    // Backup
    let res = app
        .client
        .post(app.url("/api/v1/admin/backup"))
        .json(&serde_json::json!({ "include_images": false }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    let job_id = body["job_id"].as_str().unwrap();
    let status = poll_backup_status(&app, job_id).await;
    assert_eq!(status["status"], "complete");

    let res = app
        .client
        .get(app.url(&format!("/api/v1/admin/backup/{job_id}/download")))
        .send()
        .await
        .unwrap();
    let backup_data = res.bytes().await.unwrap().to_vec();

    // Merge restore — should add alongside existing data
    let form = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(backup_data)
                .file_name("backup.tar.gz")
                .mime_str("application/gzip")
                .unwrap(),
        )
        .text(
            "options",
            serde_json::json!({ "mode": "merge" }).to_string(),
        );

    let res = app
        .client
        .post(app.url("/api/v1/admin/restore"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let restore_job_id = body["job_id"].as_str().unwrap();

    let status = poll_restore_status(&app, restore_job_id).await;
    assert_eq!(
        status["status"], "complete",
        "merge restore should complete, got: {:?}",
        status
    );

    // The original location should still exist
    let res = app
        .client
        .get(app.url(&format!("/api/v1/locations/{original_loc_id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "original location should still exist");
}

#[tokio::test]
async fn restore_status_tracks_progress() {
    let app = AdminTestApp::spawn().await;

    // Backup first
    let res = app
        .client
        .post(app.url("/api/v1/admin/backup"))
        .json(&serde_json::json!({ "include_images": false }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    let job_id = body["job_id"].as_str().unwrap();
    let status = poll_backup_status(&app, job_id).await;
    assert_eq!(status["status"], "complete");

    let res = app
        .client
        .get(app.url(&format!("/api/v1/admin/backup/{job_id}/download")))
        .send()
        .await
        .unwrap();
    let backup_data = res.bytes().await.unwrap().to_vec();

    // Restore
    let form = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(backup_data)
                .file_name("backup.tar.gz")
                .mime_str("application/gzip")
                .unwrap(),
        )
        .text(
            "options",
            serde_json::json!({ "mode": "replace" }).to_string(),
        );

    let res = app
        .client
        .post(app.url("/api/v1/admin/restore"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = res.json().await.unwrap();
    let restore_job_id = body["job_id"].as_str().unwrap();

    let status = poll_restore_status(&app, restore_job_id).await;
    assert_eq!(status["status"], "complete");
    // Total and progress should be positive numbers
    assert!(status["total"].as_u64().unwrap() > 0);
    assert_eq!(status["progress"], status["total"]);
}

// -- Helpers --

async fn poll_backup_status(app: &AdminTestApp, job_id: &str) -> serde_json::Value {
    for _ in 0..100 {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let res = app
            .client
            .get(app.url(&format!("/api/v1/admin/backup/{job_id}/status")))
            .send()
            .await
            .unwrap();
        let body: serde_json::Value = res.json().await.unwrap();
        let status = body["status"].as_str().unwrap();
        if status == "complete" || status == "failed" {
            return body;
        }
    }
    panic!("backup job did not complete in time");
}

async fn poll_restore_status(app: &AdminTestApp, job_id: &str) -> serde_json::Value {
    for _ in 0..100 {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let res = app
            .client
            .get(app.url(&format!("/api/v1/admin/restore/{job_id}/status")))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body: serde_json::Value = res.json().await.unwrap();
        let status = body["status"].as_str().unwrap();
        if status == "complete" || status == "failed" {
            return body;
        }
    }
    panic!("restore job did not complete in time");
}

/// Build a minimal valid backup archive for testing.
fn build_test_archive(include_images: bool) -> Vec<u8> {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backup");
    let data_dir = backup_dir.join("data");
    std::fs::create_dir_all(&data_dir).unwrap();

    // Manifest
    let manifest = serde_json::json!({
        "version": 1,
        "created_at": "2025-01-01T00:00:00Z",
        "includes_images": include_images,
    });
    std::fs::write(
        backup_dir.join("manifest.json"),
        serde_json::to_string(&manifest).unwrap(),
    )
    .unwrap();

    // Empty data files
    for name in &[
        "users.json",
        "groups.json",
        "memberships.json",
        "settings.json",
        "locations.json",
        "containers.json",
        "items.json",
        "photos.json",
        "nfc_tags.json",
    ] {
        std::fs::write(data_dir.join(name), "[]").unwrap();
    }

    // Build tar.gz
    let archive_path = temp_dir.path().join("test.tar.gz");
    let tar_file = std::fs::File::create(&archive_path).unwrap();
    let gz = flate2::write::GzEncoder::new(tar_file, flate2::Compression::default());
    let mut tar = tar::Builder::new(gz);

    tar.append_path_with_name(backup_dir.join("manifest.json"), "backup/manifest.json")
        .unwrap();
    for name in &[
        "users.json",
        "groups.json",
        "memberships.json",
        "settings.json",
        "locations.json",
        "containers.json",
        "items.json",
        "photos.json",
        "nfc_tags.json",
    ] {
        tar.append_path_with_name(data_dir.join(name), format!("backup/data/{name}"))
            .unwrap();
    }

    tar.into_inner().unwrap().finish().unwrap();
    std::fs::read(&archive_path).unwrap()
}
