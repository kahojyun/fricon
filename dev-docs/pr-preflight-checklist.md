# PR Preflight Checklist

## Status

Canonical pre-PR validation matrix.

## Purpose

This is the canonical pre-PR check matrix for local development and PR
readiness. Skills, agent rules, and contributing docs should link here instead
of duplicating command lists.

Use one profile per run:

- `quick` for local development loops
- `strict` once before opening or updating a PR

## Scope Detection

Check changed files first:

```bash
git diff --name-only
```

Use the result to choose changed areas:

- Rust core/CLI: `crates/fricon`, shared Rust CLI, non-frontend Rust
- Python bindings: `crates/fricon-py`, Python tests, `pyproject.toml`
- Frontend: `crates/fricon-ui/frontend`, root JS/TS config
- Tauri IPC signatures: Rust command/event changes used by UI
- Workspace compatibility: `crates/fricon/src/workspace.rs`, workspace metadata
  files, docs describing workspace layout
- Rust IPC/gRPC compatibility: `crates/fricon/proto/**`,
  `crates/fricon/src/client.rs`, `crates/fricon/src/transport/**`
- Release notes: `.changeset/**`
- Public docs: `docs/**`
- Developer docs-only: `dev-docs/**`, and Markdown-only changes outside
  `.changeset/` that are not public docs

## Quick Profile

Run only for changed areas.

### Rust

```bash
cargo +nightly fmt --all --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run
```

Optional alternative:

```bash
cargo test --workspace
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
pnpm run check
pnpm run format:check
pnpm run test
```

Optional split for diagnosis or narrow reruns:

```bash
pnpm run format:check
pnpm run type-check
pnpm run lint
pnpm run depcruise:frontend
pnpm run test:unit
pnpm run test:browser
pnpm run test:unit -- <path-or-pattern>
pnpm run test:browser -- <path-or-pattern>
```

### Tauri IPC Changed

```bash
pnpm --filter fricon-ui run gen:bindings
git diff --exit-code crates/fricon-ui/frontend/src/shared/lib/bindings.ts
```

### Workspace Compatibility Changed

Use the normal Rust test gate for coverage, then apply the workspace checklist
in `dev-docs/maintenance-checklist.md`.

### Rust IPC / gRPC Compatibility Changed

Use the normal Rust test gate for coverage, then apply the IPC/gRPC checklist in
`dev-docs/maintenance-checklist.md`.

### Frontend Router Files Changed

```bash
git diff --exit-code crates/fricon-ui/frontend/src/routeTree.gen.ts
```

### Public Docs

```bash
pnpm run format:check
uv run --group docs mkdocs build -s -v
```

### Developer Docs-Only

```bash
pnpm run format:check
```

### Release Notes

```bash
pnpm run format:check
knope --validate
```

## Strict Profile

Run once before opening or updating a PR.

### Rust

```bash
cargo +nightly fmt --all --check
cargo check
cargo build --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo nextest run --workspace --profile ci --locked
cargo test --workspace --doc --locked
cargo deny --workspace --all-features check
```

Optional alternative for Rust tests:

```bash
cargo test --workspace
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
pnpm run check
pnpm run format:check
pnpm run test
pnpm run build
git diff --exit-code crates/fricon-ui/frontend/src/routeTree.gen.ts
```

Optional split for diagnosis or narrow reruns:

```bash
pnpm run format:check
pnpm run type-check
pnpm run lint
pnpm run depcruise:frontend
pnpm run test:unit
pnpm run test:browser
pnpm run test:unit -- <path-or-pattern>
pnpm run test:browser -- <path-or-pattern>
```

### Tauri IPC Changed

```bash
pnpm --filter fricon-ui run gen:bindings
git diff --exit-code crates/fricon-ui/frontend/src/shared/lib/bindings.ts
```

### Workspace Compatibility Changed

Use the normal Rust test gate for coverage, then apply the workspace checklist
in `dev-docs/maintenance-checklist.md`.

### Rust IPC / gRPC Compatibility Changed

Use the normal Rust test gate for coverage, then apply the IPC/gRPC checklist in
`dev-docs/maintenance-checklist.md`.

### Public Docs

```bash
pnpm run format:check
uv run --group docs mkdocs build -s -v
```

### Developer Docs-Only

```bash
pnpm run format:check
```

## Environment Notes

- Local development does not need CI-style `uv sync --locked --group ci` by
  default.
- If required tools are missing locally, run once:

```bash
uv sync --all-groups
```

## Final Gate

Do not mark a PR ready if selected checks fail.

Report:

- changed area classification
- commands executed
- pass/fail result per command
- blocking failures and next fix step
- final readiness: `ready` or `not ready`
