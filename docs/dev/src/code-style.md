# Code Style

## Rust

- **Formatting**: `cargo fmt` (enforced in CI)
- **Linting**: `cargo clippy -- -D warnings` (all warnings are errors in CI)
- **Edition**: 2024, stable toolchain
- **Error handling**: `thiserror` for domain errors, `anyhow` in application code, `?` propagation
- **Async**: `async-trait` for trait methods, `tokio` runtime

## Svelte / TypeScript

- **Framework**: Svelte 5 with runes (`$state`, `$derived`, `$props`, `$effect`)
- **Type checking**: `npx tsc --noEmit` (enforced in CI)
- **Component files**: PascalCase `.svelte` files
- **Test files**: Co-located as `ComponentName.test.ts`
- **Path alias**: `~` maps to `src/` — use `import { foo } from "~/api"` not relative paths
- **Styling**: Tailwind CSS utility classes, no separate CSS files per component

## General

- Minimal, focused changes — don't refactor surrounding code
- No unnecessary comments — code should be self-explanatory
- Co-locate tests with the code they test
- Prefer simple solutions over abstractions
