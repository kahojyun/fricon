# Maintenance Checklist

## Purpose

This is the canonical maintenance checklist for coordinated repository changes.
Other docs, skills, and agent rules should link here instead of duplicating
these rules.

## General Rule

When a change crosses a user-facing boundary, update the implementation,
tests, generated artifacts, and documentation together. If a version or
compatibility decision is required, record the decision in the change.

## Workspace Format Changes

Use this when changing workspace on-disk structure, `.fricon_workspace.json`,
workspace metadata semantics, or workspace migration behavior.

- Update stepwise migration logic in `crates/fricon/src/workspace.rs`.
- Decide whether `WORKSPACE_VERSION` must change.
- Update `dev-docs/current-storage-notes.md` when the current layout or
  compatibility behavior changes.
- Update user-facing docs only when user-visible workspace behavior changes.
- Add or update Rust tests that cover migration or compatibility behavior.

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

## Database Schema Changes

Use this when changing SQLite tables, columns, indexes, constraints, Diesel
migrations, generated Diesel schema, or database model types.

- Create or update a Diesel migration under `crates/fricon/migrations`.
- Make `down.sql` a real rollback, not a placeholder.
- Preserve existing data unless the task explicitly permits destructive resets.
- Regenerate and inspect `crates/fricon/src/database/schema.rs`.
- Update affected Diesel structs, inserts/changesets, queries, and tests.
- Keep Diesel and schema types inside database/adapter code.
- Run at least `cargo check -p fricon`.
- Run targeted Rust tests for the affected database slice when present.
- Rebuild Python bindings before Python tests if exported behavior changed.

## Rust IPC / gRPC Contract Changes

Use this when changing protobuf files, transport request or response shapes,
client/server IPC behavior, or protocol compatibility semantics.

- Update the relevant files under `crates/fricon/proto/**` or
  `crates/fricon/src/transport/**`.
- Update explicit IPC protocol compatibility logic in both client and server
  where applicable.
- Decide whether `IPC_PROTOCOL_VERSION` must change.
- Regenerate affected bindings.
- Update Rust, Python, or frontend callers affected by the contract change.
- Update maintainer-facing docs when the protocol workflow or compatibility
  rule changes.
- Add or update compatibility and transport tests.

## Tauri / Frontend Binding Changes

Use this when changing Rust commands, events, DTOs, or exported Specta/Tauri
types consumed by the frontend.

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

## Python API Changes

Use this when changing Python-visible classes, methods, errors, type stubs, or
binding behavior.

- Update Rust PyO3 bindings and Python package files together.
- Update `.pyi` stubs when exported signatures or types change.
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
uv run --group docs mkdocs build -s -v
```

## Release Notes

Use this when a change is user-facing, release-note-worthy, or intentionally
changes versioned behavior.

- Add a Knope changeset file under `.changeset/`.
- AI agents should write changeset files directly rather than using the
  interactive `knope document-change` CLI.
- Human contributors may use `knope document-change` or write the file
  manually.
- Do not place templates, README files, or other helper Markdown files inside
  `.changeset/`; Knope treats them as real changesets.

Template:

```md
---
default: patch
---

# Short user-facing title

Describe the user-visible change in release-note language.
```

Use `major`, `minor`, and `patch` with standard semantic-versioning intent;
Knope handles `0.x` version behavior.
