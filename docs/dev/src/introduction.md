# Developer Guide

StoreIT is a self-hosted home inventory management system built with:

- **Backend**: Rust (edition 2024, stable toolchain), Axum web framework
- **Frontend**: Svelte 5 + Vite + Tailwind CSS, built as a PWA
- **Database**: SQLite via sqlx with compile-time checked queries
- **Auth**: OIDC (any provider) + local username/password + mock provider for dev
- **AI**: Anthropic Claude API for item identification from photos
- **Deployment**: Single binary with frontend embedded via `rust-embed`

## Quick Reference

```bash
make frontend-install   # npm install
make dev-db             # create SQLite DB + run migrations
cargo run -p storeit-server   # backend on :8080
make frontend-dev       # vite dev server on :5173 (proxies to :8080)
make test               # all Rust tests
make lint               # cargo fmt + clippy
cd frontend && npm test # frontend unit tests (Vitest)
```

## Repository Layout

```
crates/
  storeit-domain/       # Entity types, traits, errors
  storeit-db-sqlite/    # SQLite repository implementations
  storeit-storage-fs/   # Filesystem image storage + thumbnails
  storeit-auth/         # OIDC + local + mock auth
  storeit-ai/           # Anthropic API + Claude CLI
  storeit-server/       # Axum server, handlers, CLI, interchange
frontend/               # Svelte 5 + Vite + Tailwind PWA
e2e/                    # Playwright end-to-end tests
tools/                  # storeit-ctl management script
docs/                   # mdBook documentation (user + dev)
```
