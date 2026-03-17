# Upgrading

StoreIT uses a snapshot-based migration system. Upgrades work by exporting all data from the old version, then importing it into a fresh database created by the new version. This ensures clean migrations regardless of how many versions you skip.

## Using storeit-ctl (Recommended)

The `storeit-ctl` tool automates the full upgrade process:

```bash
storeit-ctl upgrade            # Upgrade to latest release
storeit-ctl upgrade 0.2.0      # Upgrade to a specific version
```

This will:
1. Back up your data via the running server's API
2. Download the new binary for your platform
3. Stop the server
4. Import data into a fresh database using the new binary
5. Start the new server
6. Run a health check

If the health check fails, it automatically rolls back to the previous version.

### After Upgrading

Verify everything works, then clean up rollback files:

```bash
storeit-ctl status     # Check version and health
storeit-ctl cleanup    # Remove .pre-import DB and old binary
```

### Rolling Back

```bash
storeit-ctl rollback
```

This restores the previous binary and database, then restarts the server.

### Downgrading

```bash
storeit-ctl downgrade 0.1.0
```

## Docker Compose

Docker upgrades are fully automatic:

```bash
docker compose pull
docker compose up -d
```

On startup, the container detects if the database schema version doesn't match the new binary. If a migration is needed, it automatically:

1. Exports all data and images from the existing database
2. Renames the old database to `.pre-upgrade`
3. Creates a fresh database with the new schema
4. Imports all data with version transforms applied
5. Starts the server

You'll see this in the container logs:

```
schema version mismatch: database has 1, binary expects 2 — auto-upgrading
exported data to temporary archive
backing up database to /data/db/storeit.db.pre-upgrade
import completed successfully
listening on 0.0.0.0:8080
```

If the migration fails, the old database is automatically restored.

### Rolling Back (Docker)

If an upgrade causes problems:

```bash
docker compose down

# The old database was saved automatically
ls ./data/db/storeit.db.pre-upgrade

# Remove the new database and restore the old one
rm ./data/db/storeit.db
mv ./data/db/storeit.db.pre-upgrade ./data/db/storeit.db

# Pin to the previous version in docker-compose.yml
# e.g., image: ghcr.io/bronson-brown-devost/storeit:0.1.0

docker compose up -d
```

### Pinning a Version

To control when you upgrade, pin to a specific version in `docker-compose.yml`:

```yaml
image: ghcr.io/bronson-brown-devost/storeit:0.1.0
```

## Manual Binary Upgrade

If you prefer to manage the process yourself:

1. Back up your data: `storeit-ctl backup -o pre-upgrade.storeit`
2. Stop the server
3. Replace the binary with the new version
4. Import: `storeit-server import pre-upgrade.storeit --mode replace`
5. Start the server

## Schema Versioning

StoreIT tracks database schema versions independently from app versions. Each binary expects a specific schema version. If there's a mismatch, the server refuses to start (or auto-upgrades in Docker).

The `import` command handles version transitions: it creates a fresh database with the new schema, applies version transforms to the data, and imports everything.

Check current versions:

```bash
storeit-server version
```
