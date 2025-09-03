# Copilot / AI agent instructions for fricon

Purpose: give an AI coding agent the minimal, actionable knowledge to be productive in this repository.

Key pointers (quick):

- Root workspace is a Rust workspace with four member crates: `crates/fricon` (core), `crates/fricon-ui` (Tauri + Vue frontend), `crates/fricon-py` (Python bindings), and `crates/fricon-cli` (CLI).
- Persistent state: a workspace directory (user-created) with files under `data/`, `log/`, and a SQLite DB at `<workspace>/fricon.sqlite3`. Use `WorkspacePaths` in `crates/fricon/src/workspace.rs` to compute any path.
- IPC & server: backend exposes gRPC over a local socket (`fricon.socket`) using `tonic` and a custom `ipc::listen` in `crates/fricon/src/ipc.rs`. Look at `crates/fricon/src/server.rs` for how services are registered.
- Database: Diesel + `deadpool-diesel` + embedded migrations. Migrations live in `crates/fricon/migrations`. Connection and migration logic in `crates/fricon/src/database.rs`.

Architecture and important files (why they matter):

- crates/fricon/src/lib.rs — high-level exports and crate purpose. Start here to find public APIs.
- crates/fricon/src/app.rs — App lifecycle: `AppManager::serve`, `AppHandle`, event broadcasting. Use this to understand how the server is started and how components get access to the workspace and DB pool.
- crates/fricon/src/server.rs — Starts gRPC server using `ipc::listen`; registers Dataset service and a Fricon service. Useful for debugging RPC surface.
- crates/fricon/src/dataset_manager.rs — Central dataset lifecycle: create, write, update, delete. Contains filesystem layout for datasets and how DB transactions are used.
- crates/fricon/src/workspace.rs — Workspace layout and locking: metadata file `.fricon_workspace.json`, exclusive lock `.fricon.lock`, and path helpers like `dataset_path_from_uuid(uuid)` (prefix = first two hex chars).
- crates/fricon/src/proto.rs & crates/fricon/proto/ — gRPC proto inclusion and token key constant.
- crates/fricon/src/database.rs & crates/fricon/migrations — DB connection, PRAGMA settings, and migration flow. The pool uses `deadpool-diesel` and runs pending migrations at startup.
- crates/fricon-ui/frontend/package.json — Frontend build/dev scripts (`vite` for dev, `vite build` for production). UI is a Tauri app (see `crates/fricon-ui/tauri.conf.json`).

Developer workflows (concrete commands & caveats):

- Setup dev environment (recommended script): `python3 scripts/setup-dev.py` — writes `.env`, creates `.dev/`, and runs `diesel setup` + migrations for the `crates/fricon` crate. The script expects `diesel` in PATH (installed via `cargo install diesel_cli --no-default-features --features sqlite`).
- Build Rust workspace: use `cargo build --workspace` (or `cargo build -p fricon` for the core crate). The workspace uses build-dependencies that generate proto code (`tonic-prost-build`) — ensure `protoc` toolchain available if editing protos.
- Run server (development): the server is started from `AppManager::serve(path)` in code, and the CLI or Tauri app triggers it. To run the headless server for a workspace, call the binary target that launches `serve` (see `crates/fricon-cli/src/main.rs`).
- Frontend dev: inside `crates/fricon-ui/frontend` run package manager commands (this repo uses pnpm at the root). Typical flow: `pnpm install` then `pnpm --filter fricon-ui/frontend dev` (or run from the directory: `pnpm dev`). The UI communicates with the backend via the IPC socket exposed by the backend.
- Tests: Rust unit/integration tests run with `cargo test`. Python integration tests present under `tests/` run with `pytest` (use the dev script `uv sync --dev` referenced in README to prepare Python environment if used).

Project-specific conventions and patterns (discoverable in code):

- Dataset storage: each dataset is stored under `<workspace>/data/<xx>/<uuid>/` where `<xx>` is the first two hex characters of the UUID (see `dataset_path_from_uuid`). Files: chunked Arrow IPC files `data_chunk_*.arrow` plus `metadata.json`.
- Dataset status values are strings in the DB (Json-backed type `DatasetStatus`) and the lifecycle follows: `pending` -> `writing` -> `completed` or `aborted`. Many functions guard by checking status (see `DatasetManager::write_dataset`).
- DB interactions: prefer `PoolExt::interact` to run synchronous Diesel queries on the pooled connection. For multi-step updates use `conn.immediate_transaction` to ensure atomic transitions (see dataset write flow).
- Eventing: app-level events are broadcast via `tokio::sync::broadcast` (see `AppEvent` in `app.rs`) — other components subscribe via `AppHandle::subscribe_to_events()`.
- IPC usage: the gRPC server uses a custom incoming stream from `ipc::listen` to serve Tonic on a local socket — don’t assume a TCP host/port when tracing RPC connections.

Integration points & external dependencies to be aware of:

- SQLite3 (development needs sqlite3 dev libs). `diesel_cli` is used for migrations in development.
- `protoc` / `tonic-prost-build` for protobuf codegen — changes to `.proto` require regenerating the Rust types (build.rs and tonic prost build take care of this during cargo builds).
- Tauri and Node toolchain for the desktop UI (pnpm, node, vite). The frontend code is in `crates/fricon-ui/frontend`.
- Python bindings: `crates/fricon-py` uses PyO3 — building Python wheels requires a matching toolchain; see `pyproject.toml` and `crates/fricon-py/Cargo.toml` for details.

Common small examples to copy-paste (search anchors in repo):

- Path helpers: use `let path = app.root().paths().dataset_path_from_uuid(uuid);` instead of constructing paths manually.
- Start server programmatically: `AppManager::serve("/path/to/workspace").await?;` (see `crates/fricon/src/app.rs`).
- Run DB work on pool: `pool.interact(|conn| /* diesel calls here */).await?;` (see `crates/fricon/src/database.rs` and usages in `dataset_manager.rs`).

What NOT to assume / gotchas:

- Do not assume a TCP gRPC endpoint; backend exposes services over a local socket via `ipc::listen`.
- Workspace directories are exclusive-locked — opening the same workspace twice fails. Use `WorkspaceRoot::validate` or `open` patterns for safe checks.
- Migrations run automatically at DB connect time; altering migration files after they have been applied locally will cause mismatch checks.

Where to look first when investigating a change or bug:

1. `crates/fricon/src/dataset_manager.rs` — dataset lifecycle and most complex logic.
2. `crates/fricon/src/app.rs` and `server.rs` — app lifecycle, eventing, server startup.
3. `crates/fricon/src/workspace.rs` — path and workspace validation/locking.
4. `crates/fricon/src/database.rs` and `crates/fricon/migrations` — DB connection and schema changes.
5. `crates/fricon-ui/frontend` — UI endpoints and how UI talks to backend (via IPC socket).

If you need more context or want the instructions tuned toward tests, UI, or Python bindings, ask which area to prioritize and I'll expand the doc with commands and examples.
