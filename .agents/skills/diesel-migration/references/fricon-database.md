# Fricon Diesel Reference

## Paths

- Diesel crate: `crates/fricon`
- Diesel config: `crates/fricon/diesel.toml`
- Migration directory: `crates/fricon/migrations`
- Generated schema: `crates/fricon/src/database/schema.rs`
- Dev database URL source: `.env`
- Dev setup helper: `scripts/setup-dev.py`

## Expected CLI Context

Interpret repository paths relative to `<project_root>`. Run Diesel CLI from `<project_root>/crates/fricon` so it picks up `diesel.toml` and writes `src/database/schema.rs`.

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
- `crates/fricon/diesel.toml` configures `print_schema.file = "src/database/schema.rs"`, so schema regeneration is part of the Diesel CLI flow.
- The current persistence code lives under `crates/fricon/src/database`. Keep new columns and tables wired into that layer, not service/business modules.
- Existing migrations are timestamped directories with `up.sql` and `down.sql`. Follow the same layout.

## Verification Checklist

After changing migrations:

1. Inspect the migration SQL diff for reversibility and data safety.
2. Inspect the generated `crates/fricon/src/database/schema.rs` diff.
3. Update affected Diesel structs, inserts/changesets, and query code.
4. Run `cargo check -p fricon`.
5. Run targeted tests for the touched database slice. Prefer `cargo nextest` when practical.
6. Rebuild Python bindings before Python tests if the schema change affects exported behavior.
