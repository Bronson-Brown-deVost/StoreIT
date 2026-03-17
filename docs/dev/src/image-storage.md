# Image Storage & Thumbnails

## Storage Backend

Images are stored on the filesystem using content-addressable storage. The `ImageStorage` trait (in `storeit-domain`) defines the interface:

```rust
#[async_trait]
pub trait ImageStorage: Send + Sync {
    async fn store(&self, name: &str, data: &[u8]) -> Result<String>;
    async fn retrieve(&self, key: &str) -> Result<(Vec<u8>, String)>;
    async fn retrieve_thumbnail(&self, key: &str) -> Result<(Vec<u8>, String)>;
    async fn delete(&self, key: &str) -> Result<()>;
}
```

## Content-Addressable Keys

Files are stored by SHA-256 hash:
- Path: `{hash[..2]}/{hash}.{extension}` (e.g., `a1/a1b2c3...d4.jpg`)
- First two hex chars as subdirectory to avoid filesystem limits

## Thumbnail Generation

Thumbnails are generated at upload time, alongside the original:

1. Decode the image using the `image` crate (supports JPEG, PNG, GIF, WebP)
2. Resize to fit within 200x200 pixels (preserving aspect ratio)
3. Encode as lossy WebP using the `webp` crate (quality 80)
4. Store as `{hash}_thumb.webp`

### Why `webp` crate, not `image` crate?

The `image` crate's built-in WebP encoder (`image-webp`) only supports **lossless VP8L** encoding. This produces degenerate output (tiny files that render as gray boxes in browsers). The `webp` crate wraps Google's `libwebp` via `libwebp-sys` and supports proper **lossy VP8** encoding.

### Fallback

If thumbnail generation fails (unsupported format, corrupted file), the full original image is served when the thumbnail is requested.

## Cache Headers

- Thumbnails: `Cache-Control: public, max-age=31536000, immutable`
- Original files: `Cache-Control: public, max-age=31536000, immutable`

These are safe because storage keys are content-addressed — if the content changes, the key changes.
