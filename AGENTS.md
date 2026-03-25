# fricon Repository Agent Rules

## Scope

- Rust crates are under `crates/`: `fricon`, `fricon-py`, `fricon-ui`, `fricon-cli`.
- Frontend is under `crates/fricon-ui/frontend`.
- `examples/` contains runnable examples.
- `scripts/` contains development helpers.
- `docs/` contains documentation sources.

## Repo-Wide Rules

- Users interact with this repo through the Python API, CLI, and desktop UI. Internal Rust APIs between crates have no stability guarantees and may be aggressively refactored or broken when it improves the architecture.
- Use non-`mod.rs` layout for Rust modules (`foo.rs` plus optional `foo/*.rs` submodules).
- Rust use nightly rustfmt: `cargo +nightly fmt`.
- Python bindings and tests (from repo root):
  run `uv run maturin develop` before `uv run pytest` when Rust bindings may be stale.
- Frontend checks (from repo root):
  use `pnpm run check` as the default frontend quality gate; run
  `pnpm run [type-check|lint|format:check|depcruise:frontend|test --run]`
  individually when needed.
- When changing workspace on-disk structure or `.fricon_workspace.json`, update
  `crates/fricon/src/workspace.rs` migration steps, decide whether
  `WORKSPACE_VERSION` must change, and update developer-facing documentation
  describing the format and maintenance duties.
- When changing Rust IPC/gRPC request or response contracts under
  `crates/fricon/proto` or `crates/fricon/src/transport`, update the explicit
  IPC protocol compatibility logic, decide whether `IPC_PROTOCOL_VERSION` must
  change, regenerate affected bindings, and document any new maintenance steps
  or constraints.
- Follow the existing vertical slice boundaries and add code within the owning domain/feature. Keep boundaries and data ownership clear, and avoid cross-feature or cross-layer shortcuts.
- Keep internal structure lightweight: prefer straightforward local implementations, feature-local duplication, and explicit types over premature shared abstractions or generic extension points.
- Add traits only for real boundaries or capabilities with multiple plausible implementations, not as a default pattern for mocking.
