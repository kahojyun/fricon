# Maintenance Checklist

## Status

Canonical maintenance checklist.

## Purpose

This is the canonical maintenance checklist for coordinated repository changes.
Other docs, skills, and agent rules should link here instead of duplicating
these rules.

## General Rule

When a change crosses a user-facing boundary, update the implementation,
tests, generated artifacts, and documentation together. If a version or
compatibility decision is required, record the decision in the change.

## Which Compatibility Surface Is Changing?

Classify the boundary before deciding which checklist applies:

| Changed surface                                                                                                    | Use                                   |
| ------------------------------------------------------------------------------------------------------------------ | ------------------------------------- |
| `.fricon_workspace.json`, workspace directory layout, durable workspace metadata, or workspace migration semantics | Workspace format changes              |
| Dataset payload files, chunking, durable dataset-side metadata, or dataset payload schema rules                    | Dataset payload layout changes        |
| Dataset archive metadata, archive entry allowlist, import/export semantics, or archive version                     | Dataset archive import/export changes |
| SQLite tables, Diesel migrations, generated Diesel schema, or database row models                                  | Database schema changes               |
| HTTP routes, event streams, binary payload endpoints, protocol files, transport modules, or IPC/gRPC semantics     | Service API / transport changes       |
| Tauri command/event DTOs, Specta exports, generated frontend bindings, or frontend-only adapter DTOs               | Tauri / frontend binding changes      |

## Workspace Format Changes

Use this when changing workspace on-disk structure, `.fricon_workspace.json`,
workspace metadata semantics, or workspace migration behavior.

- Update stepwise migration logic in `crates/fricon/src/workspace.rs`.
- Decide whether `WORKSPACE_VERSION` must change.
- Update `dev-docs/current-storage-notes.md` when the current layout or
  compatibility behavior changes.
- Update user-facing docs only when user-visible workspace behavior changes.
- Add or update Rust tests that cover migration or compatibility behavior.

Decision guide:

| Change                                                                                           | `WORKSPACE_VERSION` decision                                               |
| ------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------- |
| Add optional workspace metadata with a default and old workspaces still open without migration   | Usually no bump; document the default and add a compatibility test         |
| Add required workspace metadata or change metadata meaning so old workspaces need transformation | Bump and add a migration step                                              |
| Rename, remove, or move workspace metadata                                                       | Bump and add a migration step                                              |
| Change workspace directory layout or durable file placement                                      | Bump unless the old layout remains fully supported                         |
| SQLite-only Diesel migration with no `.fricon_workspace.json` or layout change                   | No workspace version bump; Diesel migration handles database compatibility |

Examples:

- Add optional `created_by` metadata with a default and old workspaces still
  open: usually no bump.
- Add required `workspace_uuid` metadata needed to open or interpret old
  workspaces: bump and add a migration step.

## Dataset Payload Layout Changes

Use this when changing dataset file layout, chunking behavior, dataset
directory naming, payload schema rules, or durable metadata stored beside
payload files.

- Decide whether the workspace migration path or `WORKSPACE_VERSION` must
  change.
- Update dataset read/write code and any affected recovery paths together.
- Update `dev-docs/current-storage-notes.md`.
- Update public docs only for stable user-facing behavior.
- Add or update tests for old and new payload expectations when compatibility
  matters.

## Dataset Archive Import/Export Changes

Use this when changing exported dataset archive metadata, archive entry names,
archive entry allowlists, import conflict behavior, replacement behavior, or
archive version compatibility.

- Update archive read/write code in `crates/fricon/src/dataset/portability.rs`
  together with database import/export callers.
- Decide whether the archive version must change.
- Update extraction allowlists when new archive entry types are introduced.
- Update `dev-docs/current-storage-notes.md` if current import/export storage
  behavior changes.
- Update public docs only when user-visible import/export behavior changes.
- Add or update import/export tests for backward compatibility, conflict
  handling, and rollback behavior where applicable.

Decision guide:

| Change                                                                               | Archive version decision                        |
| ------------------------------------------------------------------------------------ | ----------------------------------------------- |
| Add optional metadata that older import code may ignore                              | Usually no bump; add default-handling tests     |
| Add required metadata needed to interpret an archive                                 | Bump                                            |
| Remove, rename, or change the meaning of archive metadata                            | Bump                                            |
| Add new required archive entries or change entry naming semantics                    | Bump                                            |
| Expand the extraction allowlist for optional entries that current readers can ignore | Usually no bump; add allowlist and import tests |

## Database Schema Changes

Use this when changing SQLite tables, columns, indexes, constraints, Diesel
migrations, generated Diesel schema, or database model types.

For database fields exposed through Python, use
`dev-docs/database-schema-changes.md`.

- Create or update a Diesel migration under `crates/fricon/migrations`.
- Make `down.sql` a real rollback, not a placeholder.
- Preserve existing data unless the task explicitly permits destructive resets.
- Regenerate and inspect `crates/fricon/src/database/schema.rs`.
- Update affected Diesel structs, inserts/changesets, queries, and tests.
- Keep Diesel and schema types inside database/adapter code.
- Run at least `cargo check -p fricon`.
- Run targeted Rust tests for the affected database slice when present.
- Rebuild Python bindings before Python tests if exported behavior changed.

## Service API / Transport Contract Changes

Use this when changing HTTP routes, event streams, binary payload endpoints,
protobuf files, transport request or response shapes, client/server IPC
behavior, or protocol compatibility semantics.

The current implementation includes Rust IPC/gRPC transport. The v0.2 direction
is to converge public Fricon clients on a browser-capable HTTP/WebSocket service
API with explicit binary dataset payload endpoints. Keep this checklist focused
on the public service contract even when the implementation still has gRPC
pieces during migration.

- Update the relevant files under `crates/fricon/proto/**` or
  `crates/fricon/src/transport/**`, or the future HTTP/service API modules.
- Update explicit service API/protocol compatibility logic in both client and
  server where applicable.
- Decide whether `IPC_PROTOCOL_VERSION`, service API version, or capability
  declarations must change.
- Regenerate affected bindings.
- Update Rust, Python, or frontend callers affected by the contract change.
- Update maintainer-facing docs when the protocol workflow or compatibility
  rule changes.
- Add or update compatibility and transport tests.

Decision guide:

| Change                                                                                     | Version/capability decision                                         |
| ------------------------------------------------------------------------------------------ | ------------------------------------------------------------------- |
| Add a backward-compatible optional response field and all current clients can ignore it    | Usually no bump; add tests for old/default handling where practical |
| Add a new optional capability behind negotiation                                           | Add capability; usually no required-version bump                    |
| Add a required request field or require clients to send new data                           | Bump or require a new capability                                    |
| Remove, rename, renumber, or change the meaning/type of a public field                     | Bump                                                                |
| Change protocol handshake, stream sequencing, status semantics, or error meaning           | Bump                                                                |
| Change dataset binary payload framing, Arrow IPC expectations, or write-session sequencing | Bump or require a new capability                                    |
| Internal Rust struct refactor with no wire/protobuf/HTTP/semantic contract change          | No bump                                                             |
| Tauri/Specta-only command shape change not used by the public service API                  | No service API bump; follow the Tauri/frontend binding checklist    |

Examples:

- Add an optional protobuf response field that existing clients can ignore:
  usually no bump.
- Add a new HTTP endpoint that old clients never call: usually add a capability
  or document as a new optional feature.
- Change Arrow chunk framing or write-session finish/abort behavior: bump or
  require a new capability.
- Rename a response field or change an error/status meaning: bump.

Generated protobuf Rust code is produced by `crates/fricon/build.rs` during
Cargo builds from `crates/fricon/proto/**`. There is no checked-in protobuf
Rust output to edit. Run Rust checks/builds to regenerate in `OUT_DIR`, and
regenerate Tauri frontend bindings separately if the desktop frontend consumes
the changed shape.

## Tauri / Frontend Binding Changes

Use this when changing Rust commands, events, DTOs, or exported Specta/Tauri
types consumed by the frontend.

For cross-boundary desktop UI work, use
`dev-docs/desktop-ui-feature-playbook.md`.

- Update the owning Rust feature adapter, usually
  `crates/fricon-ui/src/features/<feature>/tauri.rs`.
- Regenerate frontend bindings:

```bash
pnpm --filter fricon-ui run gen:bindings
```

- Verify generated bindings are current:

```bash
git diff --exit-code crates/fricon-ui/frontend/src/shared/lib/bindings.ts
```

- Update frontend API adapters, query hooks, events, and tests.
- Do not hand-edit generated binding files.
- Apply `dev-docs/release-and-versioning.md` when the UI behavior is
  user-visible.

## Python API Changes

Use this when changing Python-visible classes, methods, errors, type stubs, or
binding behavior.

- Update Rust PyO3 bindings and Python package files together.
- Update `.pyi` stubs when exported signatures or types change, especially
  `crates/fricon-py/python/fricon/_core.pyi`.
- Run `uv run maturin develop` before Python tests when Rust bindings may be
  stale.
- Update Python integration tests.
- Update public docs when user-facing API behavior changes.
- Add a release note changeset when the change is user-visible.

## Frontend Route Or UI Contract Changes

Use this when changing frontend routes, route tree generation, shared UI
contracts, or user-visible screen behavior.

- Update the owning feature slice first.
- Keep feature-specific API normalization inside the feature's `api/` folder.
- Regenerate or verify generated route files when router files change:

```bash
git diff --exit-code crates/fricon-ui/frontend/src/routeTree.gen.ts
```

- Run `pnpm run check` as the default frontend gate.
- Use `pnpm run test:unit` or `pnpm run test:browser` for targeted reruns.

## Documentation Changes

Use this when changing public docs, developer docs, or repo guidance.

- Keep `docs/` user-facing.
- Put implementation details, architecture notes, and maintenance rules in
  `dev-docs/`.
- When introducing a checklist, put the canonical version in `dev-docs/` and
  link to it elsewhere.
- Run:

```bash
pnpm run format:check
uv run --group docs mkdocs build -s -v
```

## Release Notes

Use this when a change is user-facing, release-note-worthy, or intentionally
changes versioned behavior.

- Apply the policy in `dev-docs/release-and-versioning.md`.
- Add a Knope changeset file under `.changeset/` when that policy requires one.
- Record any required compatibility decision, such as `WORKSPACE_VERSION` or
  `IPC_PROTOCOL_VERSION`, in the implementation change.
