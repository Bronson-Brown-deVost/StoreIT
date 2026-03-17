# Release Process

## Versioning

StoreIT uses semantic versioning (`major.minor.patch`). The version is defined once in the workspace root:

```toml
# Cargo.toml
[workspace.package]
version = "0.1.0"
```

All crates inherit this version via `version.workspace = true`.

**Schema version** is independent — tracked in the `_meta` table as an integer (currently `1`). It only increments when the database schema changes. Defined as `SqliteDb::EXPECTED_SCHEMA_VERSION` in `crates/storeit-db-sqlite/src/lib.rs`.

## Creating a Release

1. Update version in root `Cargo.toml`
2. Commit: `git commit -m "Bump version to 0.2.0"`
3. Tag: `git tag v0.2.0`
4. Push: `git push origin main v0.2.0`
5. The release workflow builds all binaries and creates a GitHub Release with auto-generated release notes

## Release Artifacts

Each release includes:

| Asset | Description |
|-------|-------------|
| `storeit-server-linux-x86_64` | Linux x86_64 (glibc) |
| `storeit-server-linux-x86_64-musl` | Linux x86_64 (static, musl) |
| `storeit-server-linux-aarch64` | Linux ARM64 (glibc) |
| `storeit-server-linux-aarch64-musl` | Linux ARM64 (static, musl) |
| `storeit-server-darwin-aarch64` | macOS Apple Silicon |
| `storeit-server-darwin-x86_64` | macOS Intel |
| `storeit-server-windows-x86_64.exe` | Windows |
| `storeit-ctl` | Python management tool |
| `SHA256SUMS` | Checksums for all files |

## How Users Upgrade

Users run `storeit-ctl upgrade` which:
1. Backs up data via the running server's API
2. Downloads the new binary from GitHub Releases
3. Stops the server, imports data with the new binary, restarts
4. Rolls back automatically on failure

See the [migration system](./migration.md) for the technical details.

## Schema Version Changes

When adding a new migration that changes the DB schema:

1. Increment `SqliteDb::EXPECTED_SCHEMA_VERSION`
2. Add the sqlx migration file in `crates/storeit-db-sqlite/migrations/`
3. Add a transform arm in `interchange::transform_data()` for the version transition
4. Run `make prepare` to update the sqlx offline cache
