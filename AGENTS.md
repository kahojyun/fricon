# fricon Repository Agent Rules

## Monorepo Scope

- Rust crates are under `crates/`: `fricon`, `fricon-py`, `fricon-ui`, `fricon-cli`.
- `examples/` contains runnable examples.
- `scripts/` contains development helpers.
- `docs/` contains documentation sources.

## Repo-Specific Knowledge

- Use non-`mod.rs` layout for Rust modules (`foo.rs` plus optional `foo/*.rs` submodules).
- Rust use nightly rustfmt: `cargo +nightly fmt`.
- Python bindings and tests (from repo root):
  run `uv run maturin develop` before `uv run pytest` when Rust bindings may be stale.
- Frontend checks (from repo root):
  `pnpm run [type-check|lint|format:check|test --run]`.
- After Rust Tauri command/event signature changes, regenerate bindings with:
  `pnpm --filter fricon-ui run gen:bindings`.
