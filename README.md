# StoreIT

A self-hosted home inventory management system. Snap a photo, let AI identify it, and always know where everything is.

Built with Rust (Axum) and Svelte 5. Deploys as a single binary with the frontend embedded.

## Features

- **AI-powered item identification** — Take a photo and Claude identifies the item automatically
- **Hierarchical organization** — Locations > Containers > Items, with unlimited nesting
- **NFC tag support** — Tap a tag on a container to see contents or move items
- **Full-text search** — Find items by name, description, or AI-generated tags
- **Progressive Web App** — Install on any phone or tablet, works offline
- **Single-binary deployment** — No separate web server needed
- **OIDC authentication** — Integrates with Authentik or any OIDC provider

## Quick Start

### Pre-built Binary

Download the latest release for your platform from the [Releases page](https://github.com/Bronson-Brown-deVost/StoreIT/releases), then:

```bash
chmod +x storeit-server-linux-x86_64
./storeit-server-linux-x86_64
```

Open http://localhost:8080. Without an OIDC provider configured, mock auth is used automatically.

### Docker Compose

```bash
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
cd storeit
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
| `STOREIT_AUTH_ISSUER` | *(mock auth)* | OIDC issuer URL |
| `STOREIT_ANTHROPIC_API_KEY` | *(disabled)* | Anthropic API key for AI identification |
| `STOREIT_SESSION_SECRET` | *(dev default)* | Cookie signing secret (**change in production**) |

See the [full configuration reference](https://Bronson-Brown-deVost.github.io/StoreIT/user/configuration.html) for all options.

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

A multi-arch Docker image is also published.

## License

[MIT](LICENSE)
