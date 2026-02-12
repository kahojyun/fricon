# Preflight Matrix

Use one profile per run:
- `quick` for local development loops (default)
- `strict` once before opening/updating a PR

## Scope Detection

Check changed files first:
```bash
git diff --name-only
```

Use the result to choose areas:
- Rust core/CLI (`crates/fricon`, `crates/fricon-cli`, shared Rust)
- Python bindings (`crates/fricon-py`, Python tests, `pyproject.toml`)
- Frontend (`crates/fricon-ui/frontend`, root JS/TS config)
- Tauri IPC signatures (Rust command/event changes used by UI)
- Docs-only (`docs/**`, markdown files)

## Quick Profile (default)

Run only for changed areas.

### Rust
```bash
cargo +nightly fmt --all --check
cargo check
cargo clippy --all-targets --all-features
cargo test --workspace
```

Optional alternative:
```bash
cargo nextest run
```

### Python
```bash
uv run ruff format --check
uv run maturin develop
uv run ruff check
uv run pytest
```

### Frontend
```bash
pnpm run format:check
pnpm run type-check
pnpm run lint
pnpm run test --run
```

### Tauri IPC changed
```bash
pnpm --filter fricon-ui run gen:bindings
git diff --exit-code crates/fricon-ui/frontend/src/lib/bindings.ts
```

### Route tree guard (when frontend router files changed)
```bash
git diff --exit-code crates/fricon-ui/frontend/src/routeTree.gen.ts
```

### Docs-only
```bash
uv run --group docs mkdocs build -s -v
```

## Strict Profile (PR gate)

Run once before opening/updating PR.

### Rust
```bash
cargo +nightly fmt --all --check
cargo check
cargo build --workspace --locked
cargo clippy --all-targets --all-features
cargo test --workspace
cargo deny --workspace --all-features check
```

Optional alternative for Rust tests:
```bash
cargo nextest run
```

### Python
```bash
uv run ruff format --check
uv run maturin develop
uv run ruff check
uv run pytest
uv run basedpyright
uv run stubtest fricon._core
```

### Frontend
```bash
pnpm run format:check
pnpm run type-check
pnpm run lint
pnpm run test --run
pnpm run build
git diff --exit-code crates/fricon-ui/frontend/src/routeTree.gen.ts
```

### Tauri IPC changed
```bash
pnpm --filter fricon-ui run gen:bindings
git diff --exit-code crates/fricon-ui/frontend/src/lib/bindings.ts
```

### Docs
```bash
uv run --group docs mkdocs build -s -v
```

## Environment Notes

- Local development does not need CI-style `uv sync --locked --group ci` by default.
- If required tools are missing locally, run once:
```bash
uv sync --all-groups
```

## Final Gate

Do not mark PR ready if selected checks fail.
