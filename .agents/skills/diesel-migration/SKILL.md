---
name: diesel-migration
description: Use Diesel CLI to create, update, and validate SQLite schema migrations for the Fricon project. Trigger when changing database tables, columns, indexes, constraints, `crates/fricon/migrations`, `crates/fricon/src/database/schema.rs`, `crates/fricon/diesel.toml`, or when asked to maintain the project database schema with Diesel.
---

# Diesel Migration

## Overview

Use this skill for Fricon database schema work that should be driven by Diesel CLI rather than manual edits. Keep migrations reversible, treat `schema.rs` as generated output, and update only the persistence-side Rust code that owns the affected tables.

## Path Resolution

- Skill-local reference files use paths relative to this skill directory.
- Repository files and commands use paths relative to the repository root
  (`<project_root>`).

## Fricon Rules

- Run Diesel CLI from `crates/fricon`.
- Interpret repository paths relative to `<project_root>`.
- Treat `<project_root>/crates/fricon/src/database/schema.rs` as generated.
  Do not hand-edit it.
- Keep schema and Diesel model changes inside database/adapter code. Do not pull Diesel or schema modules into feature service code.
- Migrations are embedded via `embed_migrations!()`, so checked-in SQL under `crates/fricon/migrations` is the source of truth shipped in the app.
- Prefer adding a new migration for committed history. Only rewrite an in-progress migration when it is still local and safe to redo.

## Workflow

1. Read `references/fricon-database.md` for project-specific paths and
   commands, then apply the database schema section in
   `<project_root>/dev-docs/maintenance-checklist.md`.
2. Inspect the existing owning slice under `crates/fricon/src/database` before designing schema changes.
3. Create a migration with Diesel CLI:

```bash
cd <project_root>/crates/fricon
diesel migration generate <descriptive_name>
```

4. Edit the generated migration files `up.sql` and `down.sql` inside the new
   directory under `<project_root>/crates/fricon/migrations/`.
5. Apply the migration with Diesel CLI so `schema.rs` regenerates from `diesel.toml`:

```bash
cd <project_root>/crates/fricon
diesel migration run
```

6. If the migration is still local and incorrect, iterate with:

```bash
cd <project_root>/crates/fricon
diesel migration redo
```

7. Update the affected Diesel models, query code, and tests under the owning database slice.
8. Run validation commands from the repo root.

## Validation

Use the database schema checklist in
`<project_root>/dev-docs/maintenance-checklist.md`.

## When To Be Careful

- If a migration has already been committed or may be used by another checkout, do not silently rewrite it; add a follow-up migration instead.
- If `diesel` is unavailable, surface the install command rather than inventing a workaround.
- If the requested change would force Diesel or schema types into feature services, keep the schema change but contain the Rust follow-up inside adapters/persistence code.
