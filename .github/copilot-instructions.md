# Copilot instructions — fricon (condensed)

Use this file to help AI coding agents get productive quickly in this Rust/Tauri workspace.

- Top-level layout: Rust workspace with crates: `crates/fricon` (core), `crates/fricon-ui` (Tauri + Vue), `crates/fricon-py` (PyO3 bindings), `crates/fricon-cli` (CLI).

- Quick starts and important commands:
    - Dev setup: `python3 scripts/setup-dev.py` (creates `.env`, `.dev/`, runs `diesel` migrations — requires `diesel_cli` with sqlite feature).
    - Build all Rust crates: `cargo build --workspace` (or `cargo build -p fricon` for core).
    - Run frontend dev: from repo root use pnpm — `pnpm --filter fricon-ui/frontend dev` (or cd into `crates/fricon-ui/frontend` and run `pnpm dev`).

- Runtime & IPC:
    - Backend exposes gRPC over a local Unix socket (not TCP). See `crates/fricon/src/ipc.rs` and `crates/fricon/src/server.rs`.
    - Start server programmatically: `AppManager::serve(path)` in `crates/fricon/src/app.rs`.

- Database & migrations:
    - Uses SQLite + Diesel + `deadpool-diesel`. Connection & migration logic: `crates/fricon/src/database.rs`. Migrations live in `crates/fricon/migrations` and are applied at DB connect time.

- Core patterns & conventions to know:
    - Dataset layout: `<workspace>/data/<xx>/<uuid>/` (xx = first two hex chars). Files: `dataset.arrow` and `metadata.json`. See `crates/fricon/src/workspace.rs` and `crates/fricon/src/dataset_manager.rs`.
    - Lifecycle statuses: `pending` → `writing` → `completed` | `aborted`. Guard logic is common in `crates/fricon/src/dataset_manager.rs`.
    - DB access: use `pool.interact(...)` for Diesel calls; use `conn.immediate_transaction` for atomic multi-step updates.
    - Eventing: broadcast via `tokio::sync::broadcast` (see `AppEvent` in `crates/fricon/src/app.rs` and `AppHandle::subscribe_to_events`).
    - Path helpers: prefer `app.root().paths().dataset_path_from_uuid(uuid)` instead of manual path assembly.

- Integration points & build caveats:
    - Protobufs: proto sources in `crates/fricon/proto/`; build uses `tonic-prost-build` — `protoc` toolchain required when editing protos.
    - Tauri frontend requires pnpm/node/vite. The UI connects to backend via the local socket created by the server.
    - Python bindings (`crates/fricon-py`) use PyO3; building wheels may need a matching Python toolchain and cargo config.

- Where to inspect first for bugs or features:
    1. `crates/fricon/src/dataset_manager.rs` — dataset lifecycle and I/O.
    2. `crates/fricon/src/app.rs` & `crates/fricon/src/server.rs` — startup, eventing, service registration.
    3. `crates/fricon/src/workspace.rs` — workspace locking and path helpers.
    4. `crates/fricon/src/database.rs` & `crates/fricon/migrations` — DB schema and migrations.

If you want this tuned for tests, the UI, or Python bindings, tell me which area to expand.
