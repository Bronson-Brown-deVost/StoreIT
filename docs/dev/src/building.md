# Building

## Development Build

```bash
cargo run -p storeit-server    # compiles and runs
```

## Production Build

```bash
make build-all    # builds frontend, then cargo build --release
```

This produces a single binary at `target/release/storeit-server` with the frontend embedded.

### What `make build-all` Does

1. `make frontend-build` — runs `npm run build` in `frontend/`, producing `frontend/dist/`
2. `make build` — runs `make prepare` then `SQLX_OFFLINE=true cargo build --workspace --release`

The `storeit-server` crate uses `rust-embed` to embed `frontend/dist/` into the binary at compile time.

## Cross-Compilation

The project targets these platforms (via CI):

| Target | Notes |
|---|---|
| `x86_64-unknown-linux-gnu` | Linux x86_64, glibc |
| `x86_64-unknown-linux-musl` | Linux x86_64, static binary |
| `aarch64-unknown-linux-gnu` | Linux ARM64, glibc |
| `aarch64-unknown-linux-musl` | Linux ARM64, static binary |
| `x86_64-apple-darwin` | macOS Intel |
| `aarch64-apple-darwin` | macOS Apple Silicon |
| `x86_64-pc-windows-msvc` | Windows x86_64 |

The musl targets produce fully static binaries with no runtime dependencies.

Note: The `webp` crate depends on `libwebp-sys`, which compiles Google's libwebp from source. This works correctly for all targets including musl.

## Docker

```bash
docker build -t storeit .
```

The Docker image uses a multi-stage build: Rust compilation in a builder stage, then a minimal runtime image.
