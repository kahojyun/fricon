---
name: pr-preflight-checklist
description: Run a pre-pull-request quality gate for the fricon monorepo (Rust, Python bindings, and Tauri frontend). Use when preparing to open or update a PR, marking a PR ready for re-review, or when asked for PR pre-check, pre-commit check, preflight, or checklist.
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
   - run format and static checks first
   - for frontend changes, prefer `pnpm run check` as the default combined gate
   - split frontend commands only when diagnosing a failure or when you intentionally need a narrower rerun (`type-check`, `lint`, or `depcruise:frontend`)
   - build/test next
   - strict-only checks last (dependency/license checks included)
5. If the PR includes user-facing behavior changes, release-note-worthy fixes, or an intended version bump, add a Knope changeset file under `.changeset/`.
   - AI agents should write the file directly instead of using the interactive `knope document-change` CLI.
   - Human contributors may use `knope document-change` or write the file manually.
6. If Rust IPC signatures changed, run:
   - `pnpm --filter fricon-ui run gen:bindings`
   - `git diff --exit-code crates/fricon-ui/frontend/src/shared/lib/bindings.ts`
7. If workspace structure or workspace metadata semantics changed, also:
   - verify whether `WORKSPACE_VERSION` must change
   - verify docs/rules that describe workspace structure and migration duties
8. If Rust IPC/gRPC compatibility semantics changed, also:
   - verify whether `IPC_PROTOCOL_VERSION` must change
   - verify the explicit IPC protocol version/handshake logic changed with the protocol
9. Re-run failed checks after fixes, then run the selected profile once end-to-end.
10. Report results with explicit pass/fail status and any remaining risk.

## Repository Rules To Enforce

- Prefer `pnpm` and `uv` for package management commands.
- Run `uv run maturin develop` before `uv run pytest` for Python binding tests.
- Never hand-edit `crates/fricon-ui/frontend/src/shared/lib/bindings.ts`; regenerate it.
- Treat `pnpm run check` as the default frontend gate and ensure frontend slice-boundary validation is covered by it or by `pnpm run depcruise:frontend` when commands are split.
- AI agents should create Knope changeset files directly under `.changeset/` instead of using the interactive `knope document-change` command.
- Do not place templates, README files, or other helper Markdown files inside `.changeset/`; Knope treats them as real changesets. `.changeset/.gitkeep` is acceptable.
- Workspace structure changes are not complete until migration logic, the `WORKSPACE_VERSION` decision, and docs/rules are updated together.
- IPC/gRPC contract changes are not complete until the explicit IPC protocol compatibility rule, the `IPC_PROTOCOL_VERSION` decision, and generated bindings/docs are updated together.

## Knope Changeset Template

When an AI agent needs to create a Knope changeset, write a Markdown file directly in `.changeset/` using this template:

```md
---
default: patch
---

# Short user-facing title

Describe the user-visible change in release-note language.
```

Guidance:

- Use `major`, `minor`, and `patch` with standard semantic-versioning intent; Knope will handle `0.x` version behavior automatically.
- Prefer concise, user-facing wording over implementation detail.

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
