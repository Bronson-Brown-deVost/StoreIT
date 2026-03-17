# Build Pipeline

CI is defined in `.github/workflows/`.

## CI Workflow (`ci.yml`)

Runs on every push to `main` or `feature/**` branches and on pull requests.

**Jobs (run in parallel):**

| Job | What it does |
|-----|-------------|
| **Lint** | `cargo fmt --check` + `cargo clippy -D warnings` |
| **Backend Tests** | `cargo test --workspace` with `SQLX_OFFLINE=true` |
| **Frontend Tests** | `npx tsc --noEmit` + `npm test` |
| **Build Check** | Full frontend + backend release build |

## Release Workflow (`release.yml`)

Triggered by pushing a tag matching `v*` (e.g., `v0.1.0`).

### Build Matrix

| Target | Runner | Asset Name |
|--------|--------|------------|
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | `storeit-server-linux-x86_64` |
| `x86_64-unknown-linux-musl` | `ubuntu-latest` | `storeit-server-linux-x86_64-musl` |
| `aarch64-unknown-linux-gnu` | `ubuntu-latest` + `cross` | `storeit-server-linux-aarch64` |
| `aarch64-unknown-linux-musl` | `ubuntu-latest` + `cross` | `storeit-server-linux-aarch64-musl` |
| `x86_64-apple-darwin` | `macos-14` (cross from ARM) | `storeit-server-darwin-x86_64` |
| `aarch64-apple-darwin` | `macos-14` | `storeit-server-darwin-aarch64` |
| `x86_64-pc-windows-msvc` | `windows-latest` | `storeit-server-windows-x86_64.exe` |

ARM Linux targets use the [cross](https://github.com/cross-rs/cross) tool for Docker-based cross-compilation (compatible with free-tier GitHub runners).

### Release Assets

Each release includes:
- Pre-built binaries for all 7 targets
- `storeit-ctl` (Python management tool)
- `SHA256SUMS` (checksums for all files)

## Local Equivalents

```bash
make lint          # cargo fmt + clippy
make test          # cargo test --workspace
make build-all     # frontend build + cargo build --release
make coverage      # llvm-cov with 93% minimum
```
