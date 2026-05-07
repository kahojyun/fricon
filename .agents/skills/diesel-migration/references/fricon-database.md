# Fricon Diesel Reference

## Paths

- Diesel crate: `crates/fricon`
- Diesel config: `crates/fricon/diesel.toml`
- Migration directory: `crates/fricon/migrations`
- Generated schema: `crates/fricon/src/database/schema.rs`
- Dev database URL source: `.env`
- Dev setup helper: `scripts/setup-dev.py`

## Expected CLI Context

Interpret repository paths relative to `<project_root>`. Run Diesel CLI from
`<project_root>/crates/fricon` so it picks up crate-local `diesel.toml` and
writes crate-local `src/database/schema.rs`.

Typical flow:

```bash
cd <project_root>/crates/fricon
diesel migration generate <name>
diesel migration run
```

During local iteration on an unshared migration:

```bash
cd <project_root>/crates/fricon
diesel migration redo
```

If the dev database or `.env` is missing:

```bash
cd <project_root>
python3 scripts/setup-dev.py
```

If `diesel` is not installed:

```bash
cargo install diesel_cli --no-default-features --features sqlite
```

## Fricon-Specific Notes

- `crates/fricon/src/database/core.rs` embeds migrations with `embed_migrations!()`. The SQL files are shipped with the app build.
- `crates/fricon/diesel.toml` configures
  `print_schema.file = "src/database/schema.rs"` relative to
  `<project_root>/crates/fricon`, so schema regeneration is part of the Diesel
  CLI flow.
- The current persistence code lives under `crates/fricon/src/database`. Keep new columns and tables wired into that layer, not service/business modules.
- Existing migrations are timestamped directories with `up.sql` and `down.sql`. Follow the same layout.

## Verification

Use focused Rust/database validation for the changed slice. If a schema change
affects v0.2+ accepted direction, update `docs-next/` or add an ADR before
treating the migration as durable.
