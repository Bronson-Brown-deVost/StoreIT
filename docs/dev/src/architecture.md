# Architecture Overview

## High-Level Design

```
Svelte 5 PWA (embedded) ──► Axum HTTP Server
                               │
                 ┌─────────────┼─────────────┐
                 ▼             ▼             ▼
           SQLite DB    Filesystem     Claude API
           (+ FTS5)    (images)     (identification)
```

## Key Principles

### Single Binary

The Svelte frontend is compiled to static files, then embedded into the Rust binary via `rust-embed`. The result is a single executable that serves both the API and the frontend — no separate web server, no file deployment.

### Domain-Driven Design

Core types and traits live in `storeit-domain`. Repository traits define the data access interface. SQLite implementations live in `storeit-db-sqlite`. This separation means the domain logic has zero knowledge of the database.

### Compile-Time SQL

All SQL queries use `sqlx::query!` macros, which check queries against the database schema at compile time. The `.sqlx/` directory contains an offline cache so CI doesn't need a live database. Run `make prepare` after changing queries or migrations.

### Data Model

```
Location (has many)──► Container (has many)──► Item
    │                      │                     │
    ├── Photos             ├── Photos            ├── Photos
    ├── NFC Tags           ├── NFC Tags          ├── NFC Tags
    └── Child Locations    └── Child Containers  └── (leaf)
```

- Locations, containers, and items belong to a **group** for multi-tenant isolation
- Containers have polymorphic parents: either a location or another container
- Items can belong to either a container or a location directly
- Photos use content-addressable storage keyed by SHA-256 hash

### Multi-Tenant Isolation

All inventory data is scoped to a **group**. Users belong to one or more groups via memberships. API handlers receive the active group from the session and pass it to repository methods, ensuring users only see their own data.
