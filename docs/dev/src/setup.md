# Development Setup

## Prerequisites

- **Rust 1.85+** (stable toolchain)
- **Node.js 22+** and npm
- **SQLite 3**
- **sqlx-cli** — `cargo install sqlx-cli --features sqlite`

## Initial Setup

```bash
# Install frontend dependencies
make frontend-install

# Create development database and run migrations
make dev-db

# Generate sqlx offline cache
make prepare
```

## Running in Development

Run the backend and frontend dev server simultaneously:

```bash
# Terminal 1: Backend (port 8080)
cargo run -p storeit-server

# Terminal 2: Frontend dev server (port 5173, proxies API to 8080)
make frontend-dev
```

Open `http://localhost:5173` for hot-reloading frontend development. The Vite dev server proxies all `/api` requests to the Rust backend.

Without an OIDC provider configured, the mock auth provider is used automatically — any username/password works.

## Environment Variables

Copy the defaults or set these in your shell:

```bash
export DATABASE_URL="sqlite:./dev.db?mode=rwc"
export STOREIT_BIND="0.0.0.0:8080"
export STOREIT_IMAGE_PATH="./data/images"
```

See the [Configuration](../user/configuration.md) page for all options.
