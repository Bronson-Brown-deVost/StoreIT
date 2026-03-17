# Migration System

StoreIT uses a snapshot-based migration system that allows jumping between any two versions in a single step.

## Overview

Instead of incremental SQL migrations between versions, the system:

1. **Exports** all data to a portable `.storeit` archive (zstd-compressed tar)
2. **Imports** into a fresh database created by the target version's binary
3. Applies **version transforms** to the in-memory data if schema versions differ

This means any version can migrate to any other version — no chain of intermediate migrations needed.

## Key Components

### Schema Versioning (`_meta` table)

The `_meta` table stores a `schema_version` key. Each binary declares `SqliteDb::EXPECTED_SCHEMA_VERSION`. On startup, the server checks these match and refuses to start on mismatch.

```sql
CREATE TABLE _meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
INSERT INTO _meta VALUES ('schema_version', '1');
```

### Interchange Module (`interchange.rs`)

Core export/import logic, decoupled from HTTP handlers via the `ProgressReporter` trait:

- `export_to_file()` — collects all data, writes manifest v2, creates zstd tar archive
- `import_from_bytes()` — unpacks archive (zstd or gzip), parses data, applies transforms, inserts
- `transform_data()` — version transform chain (no-op for v1→v1, add match arms for future versions)

### Archive Format (`.storeit`)

Zstd-compressed tar with this structure:

```
backup/
  manifest.json          # format_version, schema_version, app_version, created_at, includes_images
  data/
    users.json           # includes _password_hash field
    groups.json
    memberships.json
    settings.json
    locations.json
    containers.json
    items.json
    photos.json          # metadata only
    nfc_tags.json
  images/                # optional
    {sha256[..2]}/{sha256}.{ext}
```

### CLI Import Command

```bash
storeit-server import archive.storeit --mode replace
```

1. Renames existing DB to `.pre-import`
2. Creates fresh DB, runs sqlx migrations
3. Reads archive, applies version transforms
4. Inserts all data, copies images
5. Rebuilds FTS5 search index
6. On failure: restores `.pre-import` back

### `storeit-ctl` Orchestrator

Python script that orchestrates the full upgrade flow:

1. Backs up via HTTP API
2. Downloads target binary from GitHub Releases
3. Stops server, swaps binary
4. Runs `storeit-server import`
5. Starts new server, health checks
6. Auto-rollback on failure

## Adding a New Schema Version

When you need to change the database schema:

1. Add a new sqlx migration in `crates/storeit-db-sqlite/migrations/`
2. Increment `SqliteDb::EXPECTED_SCHEMA_VERSION` (e.g., 1 → 2)
3. Update the migration SQL to set `schema_version` in `_meta`
4. Add a transform arm in `interchange::transform_data()`:

```rust
fn transform_data(data: &mut ArchiveData, from: i64, to: i64) -> Result<()> {
    if from == to { return Ok(()); }
    if from == 1 && to == 2 {
        // e.g., add a new field with default value
        for item in &mut data.items {
            item.new_field = Some(default_value);
        }
        return Ok(());
    }
    Err(format!("unsupported: {from} -> {to}").into())
}
```

5. Run `make prepare` to update the sqlx offline cache

## Design Decisions

- **No backward-compat gzip export**: New exports always use zstd. The HTTP restore endpoint accepts both formats (tries zstd first, falls back to gzip) for importing old archives.
- **FTS5 full reindex**: After import, `full_reindex()` clears and rebuilds the entire search index using the same `build_*_search_text` functions used at entity creation time.
- **Password hashes**: Stored in a `_password_hash` JSON field (not a DB column) so they survive export/import roundtrips.
