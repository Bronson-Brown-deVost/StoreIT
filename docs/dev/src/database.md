# Database

StoreIT uses SQLite with the sqlx library for compile-time checked queries.

## Migrations

Migrations live in `crates/storeit-db-sqlite/migrations/` and follow the naming convention `YYYYMMDDHHMMSS_description.sql`.

```bash
# Run migrations
make migrate

# Create a new migration
sqlx migrate add -r description --source crates/storeit-db-sqlite/migrations
```

## Offline Mode

sqlx checks SQL queries at compile time against a real database. For CI builds (where no database is available), an offline query cache is stored in `.sqlx/`.

```bash
# Regenerate the offline cache after changing queries
make prepare

# Build using offline cache
SQLX_OFFLINE=true cargo build
```

## Full-Text Search

StoreIT uses SQLite FTS5 for search. The FTS virtual table indexes item names, descriptions, and tags. Search queries go through the `search` repository method which joins FTS results with the main tables.

## Schema Overview

Core tables:
- `locations` — physical spaces, self-referencing for sub-locations
- `containers` — storage containers, polymorphic parent (location or container)
- `items` — tracked items, polymorphic parent
- `photos` — image metadata, polymorphic parent
- `nfc_tags` — NFC tag registrations
- `users` / `sessions` — authentication
- `settings` — application settings
- `items_fts` — FTS5 virtual table for search
