# Database Schema Changes

## Status

Canonical playbook for SQLite/Diesel schema changes, especially changes exposed
through the Python API.

## Purpose

Use this when adding or changing SQLite schema, Diesel models, dataset catalog
metadata, or database-backed behavior that becomes visible through Rust,
Python, CLI, or desktop UI surfaces.

## Read First

- `dev-docs/maintenance-checklist.md` - database and Python API checklists
- `.agents/skills/diesel-migration/SKILL.md` - Diesel CLI workflow
- `.agents/skills/diesel-migration/references/fricon-database.md` - Diesel
  paths and CLI context
- `dev-docs/architecture-guidelines.md` - persistence boundary rules
- `dev-docs/release-and-versioning.md` - changeset policy

Use the `pyo3-guide` skill only after code inspection shows PyO3 binding or
stub changes are needed.

## Current Dataset Metadata Ownership

Today, dataset catalog metadata such as name, description, favorite state,
status timestamps, and tags is stored in SQLite and mapped into
`DatasetRecord` / `DatasetMetadata`.

Dataset payload facts live in Arrow chunk files under the dataset directory.
Future semantic manifest files described by dataset semantic proposal docs are
not current behavior.

Ordinary Diesel migrations are database compatibility changes. They do not
require a `WORKSPACE_VERSION` decision unless they also change workspace
metadata files, workspace layout, or workspace migration semantics.

## Cross-Boundary Flow

For a database field visible through Python, update the layers in this order:

1. Create or update a Diesel migration under `crates/fricon/migrations`.
2. Regenerate and inspect `crates/fricon/src/database/schema.rs`.
3. Update Diesel row structs, inserts, changesets, query code, and conversions
   under `crates/fricon/src/database`.
4. Update domain/application types under `crates/fricon/src/dataset/**`, such
   as `DatasetRecord`, `DatasetMetadata`, `DatasetUpdate`, service methods, and
   repository ports.
5. Update transport/protobuf code if the field crosses gRPC/IPC boundaries.
6. Update Python bindings under `crates/fricon-py/src/`.
7. Update Python stubs under `crates/fricon-py/python/fricon/_core.pyi`.
8. Update Python tests under `crates/fricon-py/tests/`.
9. Update public docs only when user-visible behavior changes.
10. Add a changeset when the change is user-visible.

Example: adding a nullable dataset catalog field such as `operator_name` and
exposing it through Python usually touches a Diesel migration, generated
`schema.rs`, dataset catalog row structs and conversions, domain metadata
types, Python getters or update methods, `_core.pyi`, Python tests, and public
docs if users are expected to rely on the new field. It does not require a
`WORKSPACE_VERSION` bump unless workspace metadata files or layout semantics
also change.

Python-visible catalog metadata does not automatically imply a protobuf or IPC
change. Update transport contracts only when the field must travel through a
client/server request or response used by Rust, Python, or desktop callers.

## Diesel Commands

Run Diesel CLI from the crate directory:

```bash
cd <project_root>/crates/fricon
diesel migration generate <descriptive_name>
diesel migration run
```

During local iteration on an unshared migration:

```bash
cd <project_root>/crates/fricon
diesel migration redo
```

## Validation

Use targeted checks during local iteration:

Minimum local loop for a plain SQLite/catalog change:

```bash
cargo check -p fricon
cargo nextest run -p fricon
```

Add Python checks when the change is visible through Python:

```bash
uv run maturin develop
uv run pytest
uv run basedpyright
uv run stubtest fricon._core
```

If docs changed:

```bash
uv run --group docs mkdocs build -s -v
```

Before PR readiness, use `dev-docs/pr-preflight-checklist.md`.

## Common Pitfalls

- Do not hand-edit `crates/fricon/src/database/schema.rs`.
- Do not leak Diesel schema modules into service/business code.
- Do not confuse SQLite catalog metadata with Arrow payload metadata or future
  semantic manifests.
- Do not rewrite a committed migration; add a follow-up migration instead.
- Do not skip `_core.pyi` when Python-visible signatures or attributes change.
