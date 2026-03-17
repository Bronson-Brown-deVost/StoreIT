# Backup & Restore

StoreIT stores all data in two places:

1. **SQLite database** — all inventory data, user sessions, settings
2. **Image directory** — uploaded photos and generated thumbnails

## Built-in Backup (Recommended)

The Admin UI provides backup and restore under **Settings > Backup**. Backups are exported as `.storeit` archives (zstd-compressed tar) containing all data and optionally all images.

### Via the Admin UI

1. Go to **Settings > Backup**
2. Choose whether to include images
3. Click **Start Backup** and wait for completion
4. Download the `.storeit` archive

### Via storeit-ctl

```bash
# Download a full backup (data + images)
storeit-ctl backup -o my-backup.storeit
```

### Via the API

```bash
# Start a backup job
curl -X POST http://localhost:8080/api/v1/admin/backup \
  -H "Content-Type: application/json" \
  -d '{"include_images": true}'
# Returns: {"job_id": "..."}

# Poll status
curl http://localhost:8080/api/v1/admin/backup/{job_id}/status

# Download when complete
curl -o backup.storeit http://localhost:8080/api/v1/admin/backup/{job_id}/download
```

## Restore

### Via the Admin UI

1. Go to **Settings > Restore**
2. Upload a `.storeit` archive
3. Choose **Replace** (wipe and restore) or **Merge** (add alongside existing data)

### Via the CLI (offline import)

For importing into a stopped server (e.g., during upgrades):

```bash
storeit-server import backup.storeit --mode replace
```

This creates a fresh database, runs migrations, and imports all data. The previous database is saved as `storeit.db.pre-import` for rollback.

## Upgrade & Downgrade

The `storeit-ctl` tool orchestrates version migrations:

```bash
storeit-ctl upgrade              # Upgrade to latest version
storeit-ctl upgrade 0.2.0        # Upgrade to specific version
storeit-ctl downgrade 0.1.0      # Downgrade to older version
storeit-ctl rollback             # Revert if something goes wrong
storeit-ctl cleanup              # Remove rollback files after confirming
```

The upgrade flow: backup via API, download new binary, stop server, import data with new binary, start, health check. On failure, automatic rollback.

## Manual Backup

You can also back up the raw files directly:

```bash
# Stop the server (or use SQLite online backup)
sqlite3 /path/to/storeit.db ".backup '/path/to/backup/storeit.db'"

# Copy images
rsync -a /path/to/images/ /path/to/backup/images/
```

## Docker Volumes

If using Docker, back up the volume:

```bash
docker run --rm \
  -v storeit-data:/data \
  -v $(pwd)/backup:/backup \
  alpine tar czf /backup/storeit-backup.tar.gz /data
```
