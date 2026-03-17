# Backend Crates

The backend is organized as a Cargo workspace with six crates.

## `storeit-domain`

Core domain types, traits, and error definitions. No external dependencies beyond serde/chrono/uuid.

**Key types:**
- `Location`, `Container`, `Item` — entity structs
- `Photo`, `NfcTag` — supporting entities
- `LocationRepo`, `ContainerRepo`, `ItemRepo`, `PhotoRepo`, `NfcRepo` — repository traits
- `ImageStorage` — trait for image storage backends
- `AiIdentification` — AI identification result type
- `DomainError` — unified error enum

## `storeit-db-sqlite`

SQLite implementations of all repository traits using sqlx.

- Migrations in `migrations/` directory
- Uses `sqlx::query!` for compile-time SQL checking
- FTS5 virtual table for full-text search with `full_reindex()` support
- Schema versioning via `_meta` table (`schema_version()`, `set_schema_version()`)
- `EXPECTED_SCHEMA_VERSION` constant for startup version checks
- Offline query cache in `.sqlx/` (regenerate with `make prepare`)

## `storeit-storage-fs`

Filesystem-based image storage with thumbnail generation.

- Content-addressable storage: files stored as `{sha256[..2]}/{sha256}.{ext}`
- Automatic WebP thumbnail generation via `webp` crate (lossy VP8, quality 80)
- Thumbnails stored alongside originals as `{sha256}_thumb.webp`
- Max thumbnail dimension: 200px

## `storeit-auth`

Authentication providers:

- **OIDC** — Full OpenID Connect flow (Authentik, Keycloak, etc.)
- **Local** — Username/password with bcrypt hashing
- **Mock** — Accepts any credentials, for development/testing

## `storeit-ai`

AI item identification via:

- **Anthropic API** — Direct HTTP calls to Claude API with image input
- **Claude CLI** — Fallback using the `claude` CLI binary

## `storeit-server`

Axum HTTP server that wires everything together:

- Route handlers in `src/handlers/`
- Embedded frontend via `rust-embed` in `src/static_files.rs`
- OpenAPI documentation via `utoipa` (Swagger UI at `/swagger-ui`)
- 50MB body limit for photo uploads
- Integration tests in `tests/api/`
- **CLI subcommands** via clap (`src/cli.rs`): `serve`, `import <archive>`, `version`
- **Interchange module** (`src/interchange.rs`): archive export/import with zstd compression, manifest v2, version transforms
- Startup schema version check — refuses to start on mismatch

## `tools/storeit-ctl`

Python 3.8+ management script (stdlib only, no pip dependencies):

- `status` — show running version and health
- `backup` — download archive from running server
- `upgrade` / `downgrade` — orchestrate full version migration with rollback
- `install` — fresh install from GitHub Releases
- `rollback` / `cleanup` — manage rollback state
