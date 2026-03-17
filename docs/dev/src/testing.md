# Testing

## Four Test Layers

### 1. Rust Unit Tests

In each crate's `src/` files via `#[cfg(test)]` modules.

```bash
cargo test --workspace
```

### 2. Rust Integration Tests

Full HTTP API tests in `crates/storeit-server/tests/api/`. Each test spins up a real server with an in-memory SQLite database and mock auth.

```bash
cargo test -p storeit-server --test integration
```

### 3. Frontend Component Tests

Vitest with Svelte testing utilities. Co-located as `*.test.ts` next to their components.

```bash
cd frontend && npm test
```

### 4. E2E Tests

Playwright tests in `e2e/tests/`. Run against a fully built binary with mock auth and a temporary database.

```bash
make e2e-test   # builds everything first
# or manually:
cd e2e && npx playwright test
```

E2E tests block service workers (`serviceWorkers: "block"` in Playwright config) to ensure all requests hit the server directly.

## Conventions

- **TDD preferred**: Write failing tests first, then implement
- **Real images**: Photo tests use actual PNG data (via `pngjs` or hand-crafted minimal PNGs)
- **WebP validation**: Thumbnail tests verify RIFF/WEBP/VP8 headers (must be lossy VP8, not VP8L)
- **Mock auth**: Integration and E2E tests use the mock auth provider — any credentials work
- **Isolated databases**: Each test gets its own in-memory SQLite database

## Coverage

Enforced at 93% minimum via `cargo llvm-cov`:

```bash
make coverage
```

Excludes `main.rs` (CLI entry point) and `oidc.rs` (requires external provider).
