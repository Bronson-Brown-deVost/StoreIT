# StoreIT

A self-hosted home inventory management system. Snap a photo, let AI identify it, and always know where everything is.

Built with Rust (Axum) and Svelte 5. Deploys as a single binary with the frontend embedded.

## Features

- **AI-powered item identification** — Take a photo and Claude identifies the item automatically
- **Hierarchical organization** — Locations > Containers > Items, with unlimited nesting
- **NFC tag support** — Tap a tag on a container to see contents or move items
- **QR code labels** — Generate and print QR codes for any location, container, or item
- **Full-text search** — Find items by name, description, or AI-generated tags
- **Progressive Web App** — Install on any phone or tablet, works offline
- **Single-binary deployment** — No separate web server needed
- **Authentication** — Local username/password or OIDC (Authentik, Keycloak, etc.)
- **Automatic upgrades** — Snapshot-based migration system handles schema changes across versions

## Quick Start

### Using storeit-ctl (Recommended)

```bash
curl -O https://raw.githubusercontent.com/Bronson-Brown-deVost/StoreIT/main/tools/storeit-ctl
chmod +x storeit-ctl
./storeit-ctl install
```

This downloads the correct binary for your platform and sets up the data directory. Then start the server:

```bash
storeit-server
```

Open http://localhost:8080. Default login: `admin` / `changeme`.

### Docker Compose

```bash
mkdir storeit && cd storeit
curl -O https://raw.githubusercontent.com/Bronson-Brown-deVost/StoreIT/main/docker-compose.yml
curl -O https://raw.githubusercontent.com/Bronson-Brown-deVost/StoreIT/main/.env.docker
cp .env.docker .env
# Edit .env — at minimum, change STOREIT_SESSION_SECRET
docker compose up -d
```

### Build from Source

Requires Rust 1.85+, Node.js 22+, SQLite 3.

```bash
git clone https://github.com/Bronson-Brown-deVost/StoreIT.git
cd StoreIT
make frontend-install
make build-all
./target/release/storeit-server
```

## Configuration

All configuration via environment variables:

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | `sqlite:./data/storeit.db?mode=rwc` | SQLite database path |
| `STOREIT_BIND` | `0.0.0.0:8080` | Listen address |
| `STOREIT_IMAGE_PATH` | `./data/images` | Image storage directory |
| `STOREIT_AUTH_ISSUER` | *(local auth)* | OIDC issuer URL (omit for local username/password) |
| `STOREIT_ANTHROPIC_API_KEY` | *(disabled)* | Anthropic API key for AI identification |
| `STOREIT_SESSION_SECRET` | *(dev default)* | Cookie signing secret (**change in production**) |

See the [full configuration reference](https://Bronson-Brown-deVost.github.io/StoreIT/user/configuration.html) for all options.

## Upgrading

```bash
# Binary installs
storeit-ctl upgrade

# Docker
docker compose pull && docker compose up -d
```

Docker containers auto-migrate the database on startup. See the [Upgrading guide](https://Bronson-Brown-deVost.github.io/StoreIT/user/upgrading.html) for details.

## Documentation

- **[User Guide](https://Bronson-Brown-deVost.github.io/StoreIT/user/)** — Installation, usage, deployment, and administration
- **[Developer Guide](https://Bronson-Brown-deVost.github.io/StoreIT/dev/)** — Architecture, codebase, testing, and contributing
- **[API Reference](https://Bronson-Brown-deVost.github.io/StoreIT/api/)** — Interactive OpenAPI documentation

## Platform Support

CI builds binaries for:

| Platform | Variants |
|---|---|
| Linux x86_64 | glibc, musl (static) |
| Linux ARM64 | glibc, musl (static) |
| macOS | Intel, Apple Silicon |
| Windows | x86_64 |

A multi-arch Docker image (`linux/amd64`, `linux/arm64`) is also published to `ghcr.io`.

## License

[MIT](LICENSE)
