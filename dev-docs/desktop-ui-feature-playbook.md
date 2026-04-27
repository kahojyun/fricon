# Desktop UI Feature Playbook

## Status

Canonical playbook for changes that cross Rust Tauri commands/events and the
React frontend.

## Purpose

Use this when adding or changing desktop UI behavior that needs Rust-side Tauri
commands, events, exported DTOs, generated frontend bindings, or frontend
feature behavior.

## Read First

- `dev-docs/architecture-guidelines.md` - desktop UI slice boundaries
- `dev-docs/maintenance-checklist.md` - Tauri/frontend binding checklist
- `dev-docs/testing-strategy.md` - frontend test placement
- `dev-docs/release-and-versioning.md` - changeset policy for user-visible UI
  changes
- `crates/fricon-ui/AGENTS.md` - Rust UI crate constraints
- `crates/fricon-ui/frontend/AGENTS.md` - frontend slice and generated file
  rules

Do not read dataset semantic proposal files for ordinary UI action work unless
the feature directly changes dataset semantics.

## Ordered Flow

1. Identify the owning feature slice under `crates/fricon-ui/src/features/` and
   `crates/fricon-ui/frontend/src/features/`.
2. Put Rust Tauri command, event, exported DTO, native dialog, and API-error
   mapping changes in `crates/fricon-ui/src/features/<feature>/tauri.rs`.
3. Put Rust orchestration below the Tauri adapter, usually in feature-local
   modules such as `workflow.rs` only when real orchestration exists.
4. Register or export commands/events through `crates/fricon-ui/src/tauri_api.rs`
   when the global command/event set changes.
5. Update Tauri capabilities or config only when the concrete command or
   platform feature requires it. Ordinary Specta/Tauri commands usually need
   command registration and generated bindings only. Native dialog,
   filesystem, shell, window, or other platform permissions may require
   updates under:
    - `crates/fricon-ui/capabilities/`
    - `crates/fricon-ui/tauri.conf.json`
6. Regenerate frontend bindings from the repo root:

```bash
pnpm --filter fricon-ui run gen:bindings
```

7. Verify generated bindings are current:

```bash
git diff --exit-code crates/fricon-ui/frontend/src/shared/lib/bindings.ts
```

8. Keep the generic Tauri bridge in
   `crates/fricon-ui/frontend/src/shared/lib/tauri.ts` minimal.
9. Put wire-to-domain normalization, command wrappers, event subscriptions, and
   query invalidation in the frontend feature's `api/` folder.
10. Put reusable state transitions or table/action logic in `model/` or `hooks/`.
11. Put user-facing UI in `ui/`.
12. Export feature public surface through the feature barrel when app/routes
    need it.
13. Add or update tests.
14. Add a changeset when the desktop behavior is user-visible.

## Worked Example

For a dataset row or table action backed by a Rust command:

1. Put the command DTO and Tauri adapter in
   `crates/fricon-ui/src/features/datasets/tauri.rs`.
2. Register the command in `crates/fricon-ui/src/tauri_api.rs`.
3. Regenerate bindings with `pnpm --filter fricon-ui run gen:bindings`.
4. Add the frontend command wrapper and query invalidation under
   `crates/fricon-ui/frontend/src/features/datasets/api/`.
5. Put reusable action state in `model/` or `hooks/` only if more than the UI
   component needs it.
6. Add the visible row action in
   `crates/fricon-ui/frontend/src/features/datasets/ui/DatasetTableRowActions.tsx`
   or the nearest existing table-action component.
7. Add focused tests beside the changed API/model/ui code.

## Test Placement

- API normalization and query/event helpers: unit tests under `api/`.
- Reducers and table/action state transitions: unit tests under `model/`.
- Hooks that do not need real layout/focus/browser behavior: unit tests under
  `hooks/`.
- Menus, dialogs, focus, keyboard behavior, table interactions, and
  query-driven screens: browser-mode tests under `ui/` with
  `*.browser.test.*`.
- Desktop smoke tests: only for app launch, runtime, packaging, or WebView
  integration risk. Do not use smoke tests for broad component coverage.

## Validation

For local loops, prefer targeted checks:

```bash
cargo check -p fricon-ui
pnpm run check
pnpm run test:unit -- <path-or-pattern>
pnpm run test:browser -- <path-or-pattern>
```

Before PR readiness, use `dev-docs/pr-preflight-checklist.md`.

## Common Pitfalls

- Do not hand-edit generated `bindings.ts`.
- Do not import generated bindings outside `src/shared/lib/tauri.ts` and
  feature-local `api/*` modules.
- Do not let feature `workflow.rs` depend on Tauri types.
- Do not add `useMemo`, `useCallback`, or `React.memo` by default; React
  Compiler is enabled.
- Do not add shared UI components under `src/shared/ui/`; that folder is for
  shadcn primitives and thin local patches.
