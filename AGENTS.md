# fricon Repository Agent Rules

## Monorepo Scope

- Rust crates are under `crates/`: `fricon`, `fricon-py`, `fricon-ui`, `fricon-cli`.
- Frontend is under `crates/fricon-ui/frontend`.
- `examples/` contains runnable examples.
- `scripts/` contains development helpers.
- `docs/` contains documentation sources.

## Repo-Specific Knowledge

- Users interact with this repo through the Python API, CLI, and desktop UI. Internal Rust APIs between crates have no stability guarantees and may be aggressively refactored or broken when it improves the architecture.
- Use non-`mod.rs` layout for Rust modules (`foo.rs` plus optional `foo/*.rs` submodules).
- Rust use nightly rustfmt: `cargo +nightly fmt`.
- Python bindings and tests (from repo root):
  run `uv run maturin develop` before `uv run pytest` when Rust bindings may be stale.
- Frontend checks (from repo root):
  `pnpm run [type-check|lint|format:check|test --run]`.
- After Rust Tauri command/event signature changes, regenerate bindings with:
  `pnpm --filter fricon-ui run gen:bindings`.
- Run shadcn cli with `pnpm --filter fricon-ui exec shadcn`.
- `crates/fricon-ui` uses vertical feature slices.
  Rust flows as `desktop_runtime -> api::<feature> -> application::<feature> -> fricon`.
  Frontend flows as `app/routes -> features/<feature> -> feature-local api -> src/shared/lib/tauri.ts -> generated bindings`.
  Inside `frontend/src/features/**`, use relative imports only; `app` and `routes` must consume feature barrel exports.
