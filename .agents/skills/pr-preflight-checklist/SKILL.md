---
name: pr-preflight-checklist
description: Run a pre-pull-request quality gate for the fricon monorepo (Rust, Python bindings, and Tauri frontend). Use when preparing to open or update a PR, marking a PR ready for re-review, or when asked for PR pre-check, pre-commit check, preflight, or checklist.
---

# PR Preflight Checklist

## Objective

Reduce PR back-and-forth by running the smallest complete check set before pushing.

## Path Resolution

Unless explicitly stated otherwise, paths and commands in this skill are
relative to the repository root (`<project_root>`), not to this skill directory.

## Workflow

1. Identify changed scope using `git diff --name-only`.
2. Choose profile:
   - `quick` for normal local development loops (default)
   - `strict` once before opening/updating a PR
3. Map changed files to checks using the repository-root file
   `<project_root>/dev-docs/pr-preflight-checklist.md`.
4. Run selected checks in fail-fast order:
   - run format and static checks first
   - for frontend changes, prefer `pnpm run check` as the default combined gate
   - use `pnpm run test` for the full frontend batch
   - use `pnpm run test:unit` or `pnpm run test:browser` when you intentionally need one test mode
   - for targeted frontend reruns, pass file filters to `test:unit` or `test:browser` instead of the batch `test` script
   - build/test next
   - strict-only checks last (dependency/license checks included)
5. For release notes, generated bindings, workspace format changes, database
   migrations, or IPC/gRPC compatibility changes, apply the relevant section in
   the repository-root file `<project_root>/dev-docs/maintenance-checklist.md`.
6. Re-run failed checks after fixes, then run the selected profile once end-to-end.
7. Report results with explicit pass/fail status and any remaining risk.

## Repository Rules To Enforce

- Prefer `pnpm` and `uv` for package management commands.
- Run `uv run maturin develop` before `uv run pytest` for Python binding tests.
- Treat `cargo clippy --all-targets --all-features -- -D warnings` as the required Rust lint gate for preflight, not a warnings-only advisory pass.
- Never hand-edit the repository-root generated file
  `<project_root>/crates/fricon-ui/frontend/src/shared/lib/bindings.ts`;
  regenerate it.
- Treat `pnpm run check` as the default frontend gate and ensure frontend slice-boundary validation is covered by it or by `pnpm run depcruise:frontend` when commands are split.
- Treat `pnpm run test` as a batch command only; choose `test:unit` or `test:browser` for targeted reruns.
- AI agents should create Knope changeset files directly under the
  repository-root `.changeset/` directory instead of using the interactive
  `knope document-change` command.
- Do not place templates, README files, or other helper Markdown files inside
  the repository-root `.changeset/` directory; Knope treats them as real
  changesets. `.changeset/.gitkeep` is acceptable.
- Workspace, IPC/gRPC, database, generated binding, and release-note
  maintenance rules live in the repository-root file
  `<project_root>/dev-docs/maintenance-checklist.md`.

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

Repository-root files:

- `<project_root>/dev-docs/pr-preflight-checklist.md`
- `<project_root>/dev-docs/maintenance-checklist.md`
