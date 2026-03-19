# Changelog

All notable changes to StoreIT are documented in this file.

## [0.2.1] — 2026-03-19

### Added
- OpenAPI endpoint descriptions for all 54 API routes in Swagger UI
- Pre-commit git hook (`.githooks/pre-commit`) that auto-fixes `cargo fmt` and `cargo clippy` issues
- Developer documentation for git hook setup

### Fixed
- `cargo fmt` formatting violations across 4 files
- `cargo clippy` warning: useless `.into()` conversion in image resizing
- CLAUDE.md: corrected outdated references to SolidJS (now Svelte 5), schema version 1 (now 2), and test file patterns

## [0.2.0] — 2026-03-18

### Added
- Three-tier photo sizes: original, large (~1200px), and thumbnail (~200px), each content-addressed
- `GET /photos/:id/large` endpoint for display-quality images
- Lightbox photo viewer with swipe navigation and zoom
- Service worker caching for all three photo sizes (`CacheFirst`)
- Schema version 2: `thumbnail_key` and `large_key` columns on `photos` table
- Photo rotation support (`POST /photos/:id/rotate`)

### Changed
- Thumbnails and large variants generated server-side on upload (WebP, lossy VP8)
- Storage keys are now content-addressable SHA-256 hashes with shard directories

## [0.1.0] — 2026-03-17

### Added
- Initial release
- Hierarchical inventory: Locations > Containers > Items with unlimited nesting
- AI-powered item identification via Anthropic API or Claude CLI
- NFC tag support for quick container/location access
- Full-text search (SQLite FTS5)
- Photo management with multipart upload
- Progressive Web App with offline support and service worker
- Authentication: OIDC (Authentik, Keycloak, etc.) and local username/password
- Admin panel: user/group management, backup/restore, settings
- Single-binary deployment with embedded frontend (`rust-embed`)
- `storeit-ctl` management tool (Python, stdlib only)
- Docker image with auto-upgrade on startup
