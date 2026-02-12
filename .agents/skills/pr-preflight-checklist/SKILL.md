---
name: pr-preflight-checklist
description: Run a pre-pull-request quality gate for the fricon monorepo (Rust, Python bindings, and Tauri frontend). Use when preparing to open or update a PR, marking a PR ready for re-review, or when asked for PR前检查, 提交前检查, preflight, or checklist.
---

# PR Preflight Checklist

## Objective

Reduce PR back-and-forth by running the smallest complete check set before pushing.

## Workflow

1. Identify changed scope using `git diff --name-only`.
2. Choose profile:
   - `quick` for normal local development loops (default)
   - `strict` once before opening/updating a PR
3. Map changed files to checks using `references/checklist-matrix.md`.
4. Run selected checks in fail-fast order:
   - format/type/lint first
   - build/test next
   - strict-only checks last (dependency/license checks included)
5. If Rust IPC signatures changed, run:
   - `pnpm --filter fricon-ui run gen:bindings`
   - `git diff --exit-code crates/fricon-ui/frontend/src/lib/bindings.ts`
6. Re-run failed checks after fixes, then run the selected profile once end-to-end.
7. Report results with explicit pass/fail status and any remaining risk.

## Repository Rules To Enforce

- Prefer `pnpm` and `uv` for package management commands.
- Run `uv run maturin develop` before `uv run pytest` for Python binding tests.
- Never hand-edit `crates/fricon-ui/frontend/src/lib/bindings.ts`; regenerate it.

## Optional Alternatives

- If the repository is managed with Jujutsu, `jj diff --name-only` can replace `git diff --name-only`.
- If your environment uses nextest, `cargo nextest run` can replace `cargo test --workspace`.
- If tools are missing locally, run `uv sync --all-groups` once instead of CI-style group-specific syncing.

## Output Contract

Return a concise preflight summary with:

- changed area classification (Rust, Python, frontend, docs-only, mixed)
- commands executed
- pass/fail result per command
- blocking failures and next fix step
- final readiness: `ready` or `not ready`

## Reference

- `references/checklist-matrix.md`
