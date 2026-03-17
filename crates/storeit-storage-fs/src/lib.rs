use std::io::Cursor;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use image::{ImageDecoder, ImageReader};
use sha2::{Digest, Sha256};
use storeit_domain::errors::{DomainError, Result};
use storeit_domain::storage::ImageStorage;

pub struct FsImageStorage {
    base_path: PathBuf,
}

impl FsImageStorage {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    fn full_path(&self, key: &str) -> PathBuf {
        self.base_path.join(key)
    }
}

fn extension_from_mime(mime: &str) -> &str {
    match mime {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/heic" => "heic",
        _ => "bin",
    }
}

/// Compute content-addressable key: `{hex[..2]}/{hex}.{ext}`
fn compute_key(data: &[u8], mime_type: &str) -> String {
    let hash = Sha256::digest(data);
    let hex = format!("{hash:x}");
    let ext = extension_from_mime(mime_type);
    format!("{}/{hex}.{ext}", &hex[..2])
}

/// Compute thumbnail key from original key: `{hex[..2]}/{hex}_thumb.webp`
fn thumbnail_key(original_key: &str) -> String {
    let stem = original_key
        .rsplit_once('.')
        .map_or(original_key, |(s, _)| s);
    format!("{stem}_thumb.webp")
}

const THUMBNAIL_MAX_DIM: u32 = 200;

/// Generate a lossy WebP thumbnail from image bytes. Returns None if decoding fails.
/// Applies EXIF orientation so thumbnails are always right-side-up.
fn generate_thumbnail(data: &[u8]) -> Option<Vec<u8>> {
    let reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .ok()?;
    let orientation = reader
        .into_decoder()
        .ok()
        .and_then(|mut dec| dec.orientation().ok());
    // Re-read since into_decoder consumed the reader
    let reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .ok()?;
    let mut img = reader.decode().ok()?;
    if let Some(orient) = orientation {
        img.apply_orientation(orient);
    }
    let thumb = img.thumbnail(THUMBNAIL_MAX_DIM, THUMBNAIL_MAX_DIM);
    let encoder = webp::Encoder::from_image(&thumb).ok()?;
    let webp_data = encoder.encode(80.0); // lossy, quality 80
    Some(webp_data.to_vec())
}

/// Convert image bytes to lossy WebP at high quality (90%).
/// Applies EXIF orientation during conversion.
fn convert_to_webp(data: &[u8]) -> Option<Vec<u8>> {
    let reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .ok()?;
    let orientation = reader
        .into_decoder()
        .ok()
        .and_then(|mut dec| dec.orientation().ok());
    let reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .ok()?;
    let mut img = reader.decode().ok()?;
    if let Some(orient) = orientation {
        img.apply_orientation(orient);
    }
    let encoder = webp::Encoder::from_image(&img).ok()?;
    let webp_data = encoder.encode(90.0); // high quality for full-size
    Some(webp_data.to_vec())
}

fn mime_from_extension(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()) {
        Some("jpg" | "jpeg") => "image/jpeg".to_string(),
        Some("png") => "image/png".to_string(),
        Some("webp") => "image/webp".to_string(),
        Some("gif") => "image/gif".to_string(),
        Some("heic") => "image/heic".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

#[async_trait]
impl ImageStorage for FsImageStorage {
    async fn store(&self, data: &[u8], mime_type: &str) -> Result<String> {
        // Convert to WebP for storage (unless already WebP)
        let (store_data, store_mime) = if mime_type != "image/webp" {
            match convert_to_webp(data) {
                Some(webp_data) => (webp_data, "image/webp"),
                None => (data.to_vec(), mime_type), // fallback: store as-is
            }
        } else {
            (data.to_vec(), mime_type)
        };

        let key = compute_key(&store_data, store_mime);
        let path = self.full_path(&key);

        // Dedup: skip write if file already exists
        if path.exists() {
            return Ok(key);
        }

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| DomainError::Storage(e.to_string()))?;
        }
        tokio::fs::write(&path, &store_data)
            .await
            .map_err(|e| DomainError::Storage(e.to_string()))?;

        // Generate thumbnail (best-effort)
        let thumb_key = thumbnail_key(&key);
        let thumb_path = self.full_path(&thumb_key);
        if !thumb_path.exists()
            && let Some(thumb_data) = generate_thumbnail(&store_data)
        {
            if let Some(parent) = thumb_path.parent() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }
            let _ = tokio::fs::write(&thumb_path, &thumb_data).await;
        }

        Ok(key)
    }

    async fn store_at(&self, key: &str, data: &[u8], _mime_type: &str) -> Result<()> {
        let path = self.full_path(key);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| DomainError::Storage(e.to_string()))?;
        }
        tokio::fs::write(&path, data)
            .await
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    async fn retrieve(&self, key: &str) -> Result<(Vec<u8>, String)> {
        let path = self.full_path(key);
        let data = tokio::fs::read(&path)
            .await
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let mime = mime_from_extension(&path);
        Ok((data, mime))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let path = self.full_path(key);
        tokio::fs::remove_file(&path)
            .await
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    async fn delete_if_unreferenced(&self, key: &str, ref_count: i64) -> Result<bool> {
        if ref_count > 0 {
            return Ok(false);
        }
        self.delete(key).await?;
        Ok(true)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let path = self.full_path(key);
        Ok(path.exists())
    }

    async fn retrieve_thumbnail(&self, key: &str) -> Result<(Vec<u8>, String)> {
        let thumb_key = thumbnail_key(key);
        let thumb_path = self.full_path(&thumb_key);
        if thumb_path.exists() {
            let data = tokio::fs::read(&thumb_path)
                .await
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            return Ok((data, "image/webp".to_string()));
        }
        // Thumbnail missing — try to regenerate from full image
        let (full_data, mime) = self.retrieve(key).await?;
        if let Some(thumb_data) = generate_thumbnail(&full_data) {
            if let Some(parent) = thumb_path.parent() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }
            let _ = tokio::fs::write(&thumb_path, &thumb_data).await;
            return Ok((thumb_data, "image/webp".to_string()));
        }
        // Can't generate thumbnail — serve full image
        Ok((full_data, mime))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tempfile::TempDir;

    fn make_storage() -> (FsImageStorage, TempDir) {
        let dir = TempDir::new().unwrap();
        let storage = FsImageStorage::new(dir.path());
        (storage, dir)
    }

    #[tokio::test]
    async fn store_and_retrieve_roundtrip() {
        let (storage, _dir) = make_storage();
        let data = b"fake image data";
        let key = storage.store(data, "image/jpeg").await.unwrap();
        let (retrieved, mime) = storage.retrieve(&key).await.unwrap();
        assert_eq!(retrieved, data);
        assert_eq!(mime, "image/jpeg");
    }

    #[tokio::test]
    async fn store_returns_sharded_key() {
        let (storage, _dir) = make_storage();
        let key = storage.store(b"test data", "image/png").await.unwrap();
        // Key format: {hex[..2]}/{hex}.{ext}
        assert!(key.contains('/'), "key should contain shard separator");
        let parts: Vec<&str> = key.splitn(2, '/').collect();
        assert_eq!(parts[0].len(), 2, "shard prefix should be 2 hex chars");
        assert!(parts[1].ends_with(".png"), "key should end with .png");
    }

    #[tokio::test]
    async fn store_dedup_same_content() {
        let (storage, dir) = make_storage();
        let data = b"identical content";
        let key1 = storage.store(data, "image/jpeg").await.unwrap();
        let key2 = storage.store(data, "image/jpeg").await.unwrap();
        assert_eq!(key1, key2, "same content should produce same key");

        // Verify only one file on disk
        let shard_dir = dir.path().join(&key1[..2]);
        let count = std::fs::read_dir(&shard_dir).unwrap().count();
        assert_eq!(count, 1, "should be exactly one file for deduped content");
    }

    #[tokio::test]
    async fn store_at_places_file_at_exact_key() {
        let (storage, _dir) = make_storage();
        let data = b"restore data";
        storage
            .store_at("ab/somefile.jpg", data, "image/jpeg")
            .await
            .unwrap();
        let (retrieved, mime) = storage.retrieve("ab/somefile.jpg").await.unwrap();
        assert_eq!(retrieved, data);
        assert_eq!(mime, "image/jpeg");
    }

    #[tokio::test]
    async fn store_at_creates_nested_dirs() {
        let (storage, _dir) = make_storage();
        let data = b"nested";
        storage
            .store_at("a/b/c/deep.png", data, "image/png")
            .await
            .unwrap();
        let (retrieved, mime) = storage.retrieve("a/b/c/deep.png").await.unwrap();
        assert_eq!(retrieved, data);
        assert_eq!(mime, "image/png");
    }

    #[tokio::test]
    async fn exists_returns_true_after_store() {
        let (storage, _dir) = make_storage();
        let key = storage.store(b"data", "image/jpeg").await.unwrap();
        assert!(storage.exists(&key).await.unwrap());
    }

    #[tokio::test]
    async fn exists_returns_false_for_missing() {
        let (storage, _dir) = make_storage();
        assert!(!storage.exists("nope.jpg").await.unwrap());
    }

    #[tokio::test]
    async fn delete_removes_file() {
        let (storage, _dir) = make_storage();
        let key = storage.store(b"data", "image/jpeg").await.unwrap();
        storage.delete(&key).await.unwrap();
        assert!(!storage.exists(&key).await.unwrap());
    }

    #[tokio::test]
    async fn delete_if_unreferenced_skips_when_referenced() {
        let (storage, _dir) = make_storage();
        let key = storage.store(b"shared", "image/jpeg").await.unwrap();
        let deleted = storage.delete_if_unreferenced(&key, 1).await.unwrap();
        assert!(!deleted, "should not delete when ref_count > 0");
        assert!(
            storage.exists(&key).await.unwrap(),
            "file should still exist"
        );
    }

    #[tokio::test]
    async fn delete_if_unreferenced_deletes_when_zero() {
        let (storage, _dir) = make_storage();
        let key = storage.store(b"orphan", "image/jpeg").await.unwrap();
        let deleted = storage.delete_if_unreferenced(&key, 0).await.unwrap();
        assert!(deleted, "should delete when ref_count == 0");
        assert!(!storage.exists(&key).await.unwrap(), "file should be gone");
    }

    #[tokio::test]
    async fn retrieve_missing_returns_error() {
        let (storage, _dir) = make_storage();
        let result = storage.retrieve("missing.jpg").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete_missing_returns_error() {
        let (storage, _dir) = make_storage();
        let result = storage.delete("missing.jpg").await;
        assert!(result.is_err());
    }

    #[test]
    fn mime_from_extension_known_types() {
        assert_eq!(mime_from_extension(Path::new("a.jpg")), "image/jpeg");
        assert_eq!(mime_from_extension(Path::new("a.jpeg")), "image/jpeg");
        assert_eq!(mime_from_extension(Path::new("a.png")), "image/png");
        assert_eq!(mime_from_extension(Path::new("a.webp")), "image/webp");
        assert_eq!(mime_from_extension(Path::new("a.gif")), "image/gif");
        assert_eq!(mime_from_extension(Path::new("a.heic")), "image/heic");
    }

    #[tokio::test]
    async fn store_generates_thumbnail() {
        let (storage, dir) = make_storage();
        // Create a real 800x600 PNG with actual color variation
        let png_data = {
            let mut img = image::RgbImage::new(800, 600);
            for (x, y, pixel) in img.enumerate_pixels_mut() {
                *pixel = image::Rgb([(x % 256) as u8, (y % 256) as u8, 128]);
            }
            let mut buf = Cursor::new(Vec::new());
            img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
            buf.into_inner()
        };

        let key = storage.store(&png_data, "image/png").await.unwrap();
        let thumb_key = thumbnail_key(&key);
        let thumb_path = dir.path().join(&thumb_key);
        assert!(thumb_path.exists(), "thumbnail should be created on store");

        // Thumbnail should be smaller than original
        let thumb_size = std::fs::metadata(&thumb_path).unwrap().len();
        assert!(
            thumb_size < png_data.len() as u64,
            "thumbnail should be smaller"
        );

        // Thumbnail must be a valid decodable image
        let thumb_bytes = std::fs::read(&thumb_path).unwrap();
        let decoded = image::load_from_memory(&thumb_bytes)
            .expect("thumbnail must be a valid decodable image");
        assert!(decoded.width() <= THUMBNAIL_MAX_DIM);
        assert!(decoded.height() <= THUMBNAIL_MAX_DIM);
        assert!(decoded.width() > 0 && decoded.height() > 0);
    }

    #[tokio::test]
    async fn store_generates_thumbnail_from_jpeg() {
        let (storage, _dir) = make_storage();
        // Create a JPEG (the most common format from phone cameras)
        let jpeg_data = {
            let mut img = image::RgbImage::new(1024, 768);
            for (x, y, pixel) in img.enumerate_pixels_mut() {
                *pixel = image::Rgb([
                    ((x * 3 + y) % 256) as u8,
                    ((x + y * 2) % 256) as u8,
                    ((x * y) % 256) as u8,
                ]);
            }
            let mut buf = Cursor::new(Vec::new());
            img.write_to(&mut buf, image::ImageFormat::Jpeg).unwrap();
            buf.into_inner()
        };

        let key = storage.store(&jpeg_data, "image/jpeg").await.unwrap();

        // Retrieve thumbnail and verify it decodes
        let (thumb_data, mime) = storage.retrieve_thumbnail(&key).await.unwrap();
        assert_eq!(mime, "image/webp");
        let decoded = image::load_from_memory(&thumb_data)
            .expect("JPEG-sourced thumbnail must decode as valid WebP");
        assert!(decoded.width() <= THUMBNAIL_MAX_DIM);
        assert!(decoded.height() <= THUMBNAIL_MAX_DIM);
    }

    #[tokio::test]
    async fn retrieve_thumbnail_returns_thumb() {
        let (storage, _dir) = make_storage();
        let mut img = image::RgbImage::new(400, 300);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            *pixel = image::Rgb([0, (x % 256) as u8, (y % 256) as u8]);
        }
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        let png_data = buf.into_inner();

        let key = storage.store(&png_data, "image/png").await.unwrap();
        let (thumb_data, mime) = storage.retrieve_thumbnail(&key).await.unwrap();
        assert_eq!(mime, "image/webp");
        assert!(thumb_data.len() < png_data.len());

        // Must be a valid decodable image
        let decoded =
            image::load_from_memory(&thumb_data).expect("retrieved thumbnail must be decodable");
        assert!(decoded.width() > 0 && decoded.height() > 0);
    }

    #[tokio::test]
    async fn retrieve_thumbnail_falls_back_to_full() {
        let (storage, _dir) = make_storage();
        // Store raw data that can't produce a thumbnail
        let data = b"not an image";
        let key = storage.store(data, "image/jpeg").await.unwrap();
        let (retrieved, _mime) = storage.retrieve_thumbnail(&key).await.unwrap();
        assert_eq!(retrieved, data, "should fall back to full data");
    }

    #[test]
    fn thumbnail_is_valid_lossy_webp() {
        // Generate a thumbnail and verify it's a valid, displayable lossy WebP
        let mut img = image::RgbImage::new(400, 300);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            *pixel = image::Rgb([(x % 256) as u8, (y % 256) as u8, 128]);
        }
        let mut buf = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut buf, image::ImageFormat::Png)
            .unwrap();
        let png_data = buf.into_inner();

        let thumb_bytes = generate_thumbnail(&png_data).expect("should generate thumbnail");

        // Must start with RIFF/WEBP header
        assert_eq!(&thumb_bytes[..4], b"RIFF", "must start with RIFF");
        assert_eq!(&thumb_bytes[8..12], b"WEBP", "must be WEBP format");

        // Must use lossy VP8 encoding (not VP8L lossless)
        let chunk_type = &thumb_bytes[12..16];
        assert_eq!(
            chunk_type,
            b"VP8 ",
            "must be lossy VP8, got {:?}",
            std::str::from_utf8(chunk_type)
        );

        // Must be a reasonable size (not degenerate like the image crate's 178 bytes)
        assert!(
            thumb_bytes.len() > 500,
            "thumbnail must have real image data, got {} bytes",
            thumb_bytes.len()
        );

        // Verify it can be loaded back and has correct dimensions
        let decoded = image::load_from_memory(&thumb_bytes).unwrap();
        assert!(decoded.width() > 0 && decoded.width() <= THUMBNAIL_MAX_DIM);
        assert!(decoded.height() > 0 && decoded.height() <= THUMBNAIL_MAX_DIM);
    }

    #[test]
    fn thumbnail_key_format() {
        assert_eq!(thumbnail_key("ab/abc123.jpg"), "ab/abc123_thumb.webp");
        assert_eq!(thumbnail_key("cd/xyz.png"), "cd/xyz_thumb.webp");
    }

    #[test]
    fn mime_from_extension_unknown() {
        assert_eq!(
            mime_from_extension(Path::new("a.xyz")),
            "application/octet-stream"
        );
        assert_eq!(
            mime_from_extension(Path::new("noext")),
            "application/octet-stream"
        );
    }

    #[test]
    fn compute_key_format() {
        let key = compute_key(b"hello", "image/jpeg");
        let parts: Vec<&str> = key.splitn(2, '/').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].len(), 2);
        assert!(parts[1].ends_with(".jpg"));
        // SHA-256 of "hello" starts with 2cf24d
        assert!(key.starts_with("2c/2cf24d"));
    }
}
