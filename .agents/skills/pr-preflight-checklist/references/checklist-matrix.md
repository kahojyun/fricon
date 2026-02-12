# Preflight Matrix

Use this matrix to select the minimum complete check set based on changed files.

## 1) Docs-only changes

Trigger:
- `docs/**`
- `README.md`
- markdown-only updates

Run:
```bash
uv run --group docs mkdocs build -s -v
cargo check
```

## 2) Rust core and CLI (`crates/fricon`, `crates/fricon-cli`, shared Rust code)

Run:
```bash
cargo +nightly fmt --all --check
cargo check
cargo build --workspace --locked
cargo clippy --all-targets --all-features
cargo test --workspace
```

Optional alternative:
```bash
cargo nextest run
```

## 3) Python bindings (`crates/fricon-py`, Python tests, pyproject files)

Run:
```bash
uv run ruff format --check
uv run maturin develop
uv sync --locked --group ci
uv run ruff check
uv run pytest
uv run basedpyright
uv run stubtest fricon._core
```

## 4) Frontend/UI (`crates/fricon-ui/frontend`, root `package.json`, TS/ESLint config)

Run:
```bash
pnpm run format:check
pnpm run type-check
pnpm run lint
git diff --exit-code crates/fricon-ui/frontend/src/routeTree.gen.ts
pnpm run test --run
pnpm run build
```

## 5) Tauri IPC or Rust command signature changes

Trigger:
- Rust command/event changes used by UI
- changes around `tauri-specta` definitions

Run additionally:
```bash
pnpm --filter fricon-ui run gen:bindings
git diff --exit-code crates/fricon-ui/frontend/src/lib/bindings.ts
pnpm run format:check
pnpm run type-check
pnpm run lint
git diff --exit-code crates/fricon-ui/frontend/src/routeTree.gen.ts
```

## 6) Mixed changes (Rust + Python + frontend)

Run all relevant sections, in this order:
1. `cargo +nightly fmt --all --check`
2. `uv run ruff format --check`
3. `pnpm run format:check`
4. `cargo check`
5. `cargo build --workspace --locked`
6. `uv run maturin develop`
7. `uv sync --locked --group ci`
8. `uv run ruff check`
9. `pnpm run type-check`
10. `pnpm run lint`
11. `cargo clippy --all-targets --all-features`
12. `pnpm --filter fricon-ui run gen:bindings` (if IPC changed)
13. `git diff --exit-code crates/fricon-ui/frontend/src/lib/bindings.ts` (if IPC changed)
14. `git diff --exit-code crates/fricon-ui/frontend/src/routeTree.gen.ts`
15. `cargo test --workspace`
16. `uv run pytest`
17. `uv run basedpyright`
18. `uv run stubtest fricon._core`
19. `pnpm run test --run`
20. `pnpm run build`
21. `uv run --group docs mkdocs build -s -v`

Optional alternative for step 15:
- `cargo nextest run`

## Final Gate

Do not mark PR ready if any selected command fails.
