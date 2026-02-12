---
name: pr-preflight-checklist
description: Run a pre-pull-request quality gate for the fricon monorepo (Rust, Python bindings, and Tauri frontend). Use when preparing to open or update a PR, marking a PR ready for re-review, or when asked for PR前检查, 提交前检查, preflight, or checklist.
---

# PR Preflight Checklist

## Objective

Reduce PR back-and-forth by running the smallest complete check set before pushing.

## Workflow

1. Identify changed scope using `git diff --name-only`.
2. Map changed files to checks using `references/checklist-matrix.md`.
3. Run checks in fail-fast order:
   - static and type/lint checks first
   - build/compile checks second
   - tests last
4. Regenerate frontend bindings when Rust IPC signatures changed:
   - `pnpm --filter fricon-ui run gen:bindings`
5. Re-run only failed commands after fixes, then run the full selected set once.
6. Report results with explicit pass/fail status and any remaining risk.

## Repository Rules To Enforce

- Prefer `pnpm` and `uv` for package management commands.
- Run `uv run maturin develop` before `uv run pytest` for Python binding tests.
- Never hand-edit `crates/fricon-ui/frontend/src/lib/bindings.ts`; regenerate it.

## Optional Alternatives

- If the repository is managed with Jujutsu, `jj diff --name-only` can replace `git diff --name-only`.
- If your environment uses nextest, `cargo nextest run` can replace `cargo test --workspace`.

## Output Contract

Return a concise preflight summary with:

- changed area classification (Rust, Python, frontend, docs-only, mixed)
- commands executed
- pass/fail result per command
- blocking failures and next fix step
- final readiness: `ready` or `not ready`

## Reference

- `references/checklist-matrix.md`
