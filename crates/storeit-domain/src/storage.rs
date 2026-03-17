use async_trait::async_trait;

use crate::entities::IdentificationResult;
use crate::errors::Result;

/// Pluggable image storage backend.
#[async_trait]
pub trait ImageStorage: Send + Sync {
    /// Store data; returns the content-addressable key (hash-based).
    async fn store(&self, data: &[u8], mime_type: &str) -> Result<String>;
    /// Store data at a specific key (for restore/migration). Does not compute hash.
    async fn store_at(&self, key: &str, data: &[u8], mime_type: &str) -> Result<()>;
    /// Returns `(bytes, mime_type)`.
    async fn retrieve(&self, key: &str) -> Result<(Vec<u8>, String)>;
    /// Delete unconditionally.
    async fn delete(&self, key: &str) -> Result<()>;
    /// Delete only if ref_count == 0. Returns true if deleted.
    async fn delete_if_unreferenced(&self, key: &str, ref_count: i64) -> Result<bool>;
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Retrieve a thumbnail for the given key, if one exists.
    /// Returns `(bytes, mime_type)`. Falls back to the full image if no thumbnail.
    async fn retrieve_thumbnail(&self, key: &str) -> Result<(Vec<u8>, String)> {
        self.retrieve(key).await
    }
}

/// AI item identification — define trait now, implement in M3.
#[async_trait]
pub trait ItemIdentifier: Send + Sync {
    async fn identify(&self, image_data: &[u8], mime_type: &str) -> Result<IdentificationResult>;
    async fn identify_with_correction(
        &self,
        image_data: &[u8],
        mime_type: &str,
        correction: &str,
    ) -> Result<IdentificationResult>;
}
