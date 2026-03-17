# API Reference

The full API is documented via OpenAPI 3.0 and published automatically on every push to `main`.

## Interactive Documentation

**[View the API Reference](https://Bronson-Brown-deVost.github.io/StoreIT/api/)**

This is a live Swagger UI loaded from the auto-generated OpenAPI spec. Every endpoint, request body, and response schema is documented and testable.

## Local Swagger UI

When running the server locally, the same documentation is available at:

```
http://localhost:8080/swagger-ui
```

The local version lets you make real API calls against your running instance.

## How It Works

All handlers use `#[utoipa::path(...)]` annotations that generate the OpenAPI spec at compile time. The CI docs workflow:

1. Builds the server
2. Starts it briefly with an in-memory database
3. Fetches `/api-docs/openapi.json`
4. Publishes it alongside a static Swagger UI page

This means the published API docs always match the code on `main` — no manual sync needed.

## Endpoint Organization

All endpoints are under `/api/v1/`:

| Prefix | Module | Description |
|--------|--------|-------------|
| `/locations` | `handlers/locations.rs` | Location CRUD + tree |
| `/containers` | `handlers/containers.rs` | Container CRUD + move |
| `/items` | `handlers/items.rs` | Item CRUD + batch + move |
| `/photos` | `handlers/photos.rs` | Upload, serve, thumbnails, rotate |
| `/search` | `handlers/search.rs` | Full-text search |
| `/nfc-tags` | `handlers/nfc.rs` | NFC tag management + resolution |
| `/identify` | `handlers/identify.rs` | AI item identification |
| `/auth` | `handlers/auth.rs` | OIDC + local login + sessions |
| `/admin` | `handlers/admin.rs` | Users, groups, settings, backup/restore |

## Handler Conventions

- Use Axum extractors: `State`, `Path`, `Query`, `Json`, `Multipart`
- All handlers return `Result<impl IntoResponse, AppError>`
- `AppError` wraps `DomainError` and maps to HTTP status codes
- OpenAPI annotations via `#[utoipa::path(...)]`
- Routes registered with `routes!()` macro from `utoipa-axum`
