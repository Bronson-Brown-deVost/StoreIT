use crate::common::AdminTestApp;

#[tokio::test]
async fn backup_requires_admin() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .non_admin_client
        .post(app.url("/api/v1/admin/backup"))
        .json(&serde_json::json!({ "include_images": false }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn backup_unknown_job_returns_404() {
    let app = AdminTestApp::spawn().await;

    let res = app
        .client
        .get(app.url("/api/v1/admin/backup/nonexistent-id/status"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn backup_data_only_produces_tar_gz() {
    let app = AdminTestApp::spawn().await;

    // Start backup
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

    // Poll until complete
    let status = poll_backup_status(&app, job_id).await;
    assert_eq!(status["status"], "complete");

    // Download the archive
    let res = app
        .client
        .get(app.url(&format!("/api/v1/admin/backup/{job_id}/download")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    assert!(
        res.headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("octet-stream")
    );
    assert!(
        res.headers()
            .get("content-disposition")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("storeit-backup-")
    );

    let bytes = res.bytes().await.unwrap();
    assert!(!bytes.is_empty());

    // Verify archive contents (zstd-compressed tar)
    let cursor = std::io::Cursor::new(&bytes);
    let decoder = zstd::stream::read::Decoder::new(cursor).unwrap();
    let mut archive = tar::Archive::new(decoder);

    let entries: Vec<String> = archive
        .entries()
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path().unwrap().to_string_lossy().to_string())
        .collect();

    assert!(entries.contains(&"backup/manifest.json".to_string()));
    assert!(entries.contains(&"backup/data/users.json".to_string()));
    assert!(entries.contains(&"backup/data/groups.json".to_string()));
    assert!(entries.contains(&"backup/data/locations.json".to_string()));
    assert!(entries.contains(&"backup/data/containers.json".to_string()));
    assert!(entries.contains(&"backup/data/items.json".to_string()));
    assert!(entries.contains(&"backup/data/photos.json".to_string()));
    assert!(entries.contains(&"backup/data/nfc_tags.json".to_string()));
    assert!(entries.contains(&"backup/data/settings.json".to_string()));
    assert!(entries.contains(&"backup/data/memberships.json".to_string()));
    // No images directory expected
    assert!(!entries.iter().any(|e| e.starts_with("backup/images/")));
}

#[tokio::test]
async fn backup_status_tracks_progress() {
    let app = AdminTestApp::spawn().await;

    // Start backup
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
    // total should be 9 (9 data files, no images)
    assert_eq!(status["total"], 9);
    // progress should equal total when complete
    assert_eq!(status["progress"], status["total"]);
}

#[tokio::test]
async fn backup_with_images_includes_image_files() {
    let app = AdminTestApp::spawn().await;

    // Upload a photo first — we need a location to attach it to
    let loc_res = app
        .client
        .post(app.url("/api/v1/locations"))
        .json(&serde_json::json!({ "name": "Test Room" }))
        .send()
        .await
        .unwrap();
    assert_eq!(loc_res.status(), 201);
    let loc: serde_json::Value = loc_res.json().await.unwrap();
    let loc_id = loc["id"].as_str().unwrap();

    // Upload a 1x1 PNG
    let png_data = tiny_png();
    let part = reqwest::multipart::Part::bytes(png_data)
        .file_name("test.png")
        .mime_str("image/png")
        .unwrap();
    let form = reqwest::multipart::Form::new()
        .text("entity_type", "location")
        .text("entity_id", loc_id.to_string())
        .part("file", part);

    let photo_res = app
        .client
        .post(app.url("/api/v1/photos"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(photo_res.status(), 201);

    // Now backup with images
    let res = app
        .client
        .post(app.url("/api/v1/admin/backup"))
        .json(&serde_json::json!({ "include_images": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let job_id = body["job_id"].as_str().unwrap();

    let status = poll_backup_status(&app, job_id).await;
    assert_eq!(status["status"], "complete");
    // total should be 9 data files + 1 image
    assert_eq!(status["total"], 10);

    // Download and verify images are included
    let res = app
        .client
        .get(app.url(&format!("/api/v1/admin/backup/{job_id}/download")))
        .send()
        .await
        .unwrap();
    let bytes = res.bytes().await.unwrap();
    let cursor = std::io::Cursor::new(&bytes);
    let decoder = zstd::stream::read::Decoder::new(cursor).unwrap();
    let mut archive = tar::Archive::new(decoder);

    let entries: Vec<String> = archive
        .entries()
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path().unwrap().to_string_lossy().to_string())
        .collect();

    assert!(
        entries.iter().any(|e| e.starts_with("backup/images/")),
        "archive should contain images, entries: {:?}",
        entries
    );
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
        assert_eq!(res.status(), 200);
        let body: serde_json::Value = res.json().await.unwrap();
        let status = body["status"].as_str().unwrap();
        if status == "complete" || status == "failed" {
            return body;
        }
    }
    panic!("backup job did not complete in time");
}

/// Generate a minimal valid PNG (1x1 red pixel).
fn tiny_png() -> Vec<u8> {
    let mut buf = Vec::new();
    // PNG signature
    buf.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    // Helper to write a PNG chunk
    fn write_chunk(buf: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
        buf.extend_from_slice(&(data.len() as u32).to_be_bytes());
        buf.extend_from_slice(chunk_type);
        buf.extend_from_slice(data);
        // CRC over chunk_type + data
        let mut crc_data = Vec::with_capacity(4 + data.len());
        crc_data.extend_from_slice(chunk_type);
        crc_data.extend_from_slice(data);
        let crc = crc32_calc(&crc_data);
        buf.extend_from_slice(&crc.to_be_bytes());
    }

    // IHDR chunk
    let ihdr_data: [u8; 13] = [
        0, 0, 0, 1, // width
        0, 0, 0, 1, // height
        8, // bit depth
        2, // color type (RGB)
        0, 0, 0, // compression, filter, interlace
    ];
    write_chunk(&mut buf, b"IHDR", &ihdr_data);

    // IDAT chunk — zlib-compressed scanline: [filter=0, R=255, G=0, B=0]
    let raw_scanline = [0u8, 255, 0, 0];
    let compressed = {
        use flate2::Compression;
        use flate2::write::ZlibEncoder;
        use std::io::Write;
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&raw_scanline).unwrap();
        encoder.finish().unwrap()
    };
    write_chunk(&mut buf, b"IDAT", &compressed);

    // IEND chunk
    write_chunk(&mut buf, b"IEND", &[]);

    buf
}

fn crc32_calc(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}
