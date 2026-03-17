# Installation

## Using storeit-ctl (Recommended)

`storeit-ctl` is a single Python script (Python 3.8+, no pip dependencies) that handles downloading, installing, and managing StoreIT.

```bash
# Download storeit-ctl
curl -O https://raw.githubusercontent.com/Bronson-Brown-deVost/StoreIT/main/tools/storeit-ctl
chmod +x storeit-ctl

# Install the latest version
./storeit-ctl install
```

This downloads the correct binary for your platform from GitHub Releases, places it at `/usr/local/bin/storeit-server`, and creates the data directory.

Start the server:

```bash
storeit-server
```

Open http://localhost:8080. Default login: `admin` / `changeme`.

### Configuration

Configure `storeit-ctl` via environment variables or `~/.config/storeit/ctl.conf`:

```bash
export STOREIT_CTL_SERVER_URL=http://localhost:8080
export STOREIT_CTL_BINARY_PATH=/usr/local/bin/storeit-server
export STOREIT_CTL_DATA_DIR=/var/lib/storeit
```

## Docker Compose

For Docker users, see [Deployment > Docker Compose](./deployment.md#docker-compose-recommended-for-docker).

```bash
mkdir storeit && cd storeit
curl -O https://raw.githubusercontent.com/Bronson-Brown-deVost/StoreIT/main/docker-compose.yml
curl -O https://raw.githubusercontent.com/Bronson-Brown-deVost/StoreIT/main/.env.docker
cp .env.docker .env
# Edit .env — at minimum, change STOREIT_SESSION_SECRET
docker compose up -d
```

## Manual Binary Install

Download the latest release for your platform from the [Releases page](https://github.com/Bronson-Brown-deVost/StoreIT/releases):

| Platform | File |
|---|---|
| Linux x86_64 (glibc) | `storeit-server-linux-x86_64` |
| Linux x86_64 (musl, static) | `storeit-server-linux-x86_64-musl` |
| Linux ARM64 (glibc) | `storeit-server-linux-aarch64` |
| Linux ARM64 (musl, static) | `storeit-server-linux-aarch64-musl` |
| macOS (Apple Silicon) | `storeit-server-darwin-aarch64` |
| macOS (Intel) | `storeit-server-darwin-x86_64` |
| Windows | `storeit-server-windows-x86_64.exe` |

```bash
chmod +x storeit-server-linux-x86_64
./storeit-server-linux-x86_64
```

The server starts on `http://localhost:8080` by default.

## Build from Source

Requirements: Rust 1.85+, Node.js 22+, SQLite 3.

```bash
git clone https://github.com/Bronson-Brown-deVost/StoreIT.git
cd StoreIT
make frontend-install
make build-all
./target/release/storeit-server
```
