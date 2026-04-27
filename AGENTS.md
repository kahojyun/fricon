# fricon Repository Agent Rules

## Scope

- Rust crates are under `crates/`: `fricon`, `fricon-py`, `fricon-ui`.
- Frontend is under `crates/fricon-ui/frontend`.
- `examples/` contains runnable examples.
- `scripts/` contains development helpers.
- `docs/` contains public user-facing documentation sources.
- `dev-docs/` contains internal developer notes, implementation details,
  architecture notes, and maintenance guidance that should not be published as
  user-facing docs.

## Repo-Wide Rules

- Use `python3 scripts/setup-dev.py` only when you need the dev workspace
  created through `fricon init`.
- Keep setup lazy: run `uv sync`, `uv run maturin develop`, `pnpm install`, and
  broader builds only when the task actually needs them.
- Users interact with this repo through the Python API, CLI, and desktop UI. Internal Rust APIs between crates have no stability guarantees and may be aggressively refactored or broken when it improves the architecture.
- Use non-`mod.rs` layout for Rust modules (`foo.rs` plus optional `foo/*.rs` submodules).
- Rust use nightly rustfmt: `cargo +nightly fmt`.
- Python bindings and tests (from repo root):
  run `uv run maturin develop` before `uv run pytest` when Rust bindings may be stale.
- Frontend checks (from repo root):
  use `pnpm run check` as the default quality gate.
  Use `pnpm run test` for the full frontend batch,
  `pnpm run test:unit` for unit/jsdom tests,
  `pnpm run test:browser` for browser-mode tests, and
  `pnpm run test:smoke` for the desktop smoke suite.
  For targeted reruns, call `test:unit` or `test:browser` directly instead of
  passing file filters through the batch `test` script.
  Desktop smoke is CI-validated on Windows only; Tauri does not provide desktop
  WebDriver support on macOS.
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
