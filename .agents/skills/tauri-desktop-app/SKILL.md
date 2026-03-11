---
name: tauri-desktop-app
description: Repo-specific guidance for `crates/fricon-ui` Tauri work. Use when changing Rust commands/events, `tauri-specta` bindings, tray/window lifecycle, or Tauri config/capability files.
---

# Tauri Desktop App

Use this skill only for work under `crates/fricon-ui` that crosses the Rust/Tauri boundary.

## Read First

- Rust Tauri contract and binding export: `crates/fricon-ui/src/api.rs`
- Feature command and event modules: `crates/fricon-ui/src/api/<feature>.rs`
- Application orchestration: `crates/fricon-ui/src/application/<feature>.rs`
- App setup, tray, window lifecycle: `crates/fricon-ui/src/desktop_runtime/runtime.rs`
- Tauri config: `crates/fricon-ui/tauri.conf.json`
- Capabilities and permissions: `crates/fricon-ui/capabilities/default.json`
- Frontend Tauri helper: `crates/fricon-ui/frontend/src/shared/lib/tauri.ts`
- Frontend feature slices: `crates/fricon-ui/frontend/src/features/<feature>/`
- Generated bindings: `crates/fricon-ui/frontend/src/shared/lib/bindings.ts`

## Repo Rules

- This repo uses Tauri v2 with `tauri-specta`; Rust is the type source of truth for exported commands, payloads, and events.
- Do not edit `crates/fricon-ui/frontend/src/shared/lib/bindings.ts` manually.
- Regenerate bindings with `pnpm --filter fricon-ui run gen:bindings` after Rust command or event signature changes.
- Keep the generic frontend Tauri bridge minimal in `crates/fricon-ui/frontend/src/shared/lib/tauri.ts`.
- Keep wire-to-domain normalization, query keys, and event invalidation in feature-local frontend `api/` modules.
- Keep the Rust dependency flow one-way: `desktop_runtime -> api::<feature> -> application::<feature> -> fricon`.
- Keep the frontend dependency flow one-way: `app/routes -> feature barrels -> feature-local api -> shared/lib/tauri.ts -> generated bindings`.
- Inside `frontend/src/features/**`, use relative imports only. `app` and `routes` should consume feature barrel exports, not feature internals.
- When a feature needs Tauri capabilities or plugin permissions, update `crates/fricon-ui/capabilities/default.json`.
- Keep CI `fmt` lightweight. Do not move binding generation or Rust-compiling checks into `fmt`.

## Workflow

1. Decide whether the change is frontend-only or needs a Rust command/event.
2. If IPC changes, update the Rust feature module first in `crates/fricon-ui/src/api/<feature>.rs`.
3. If application behavior changes, inspect the matching `crates/fricon-ui/src/application/<feature>.rs` module before adding transport code.
4. If runtime behavior changes, inspect `crates/fricon-ui/src/desktop_runtime/runtime.rs` before adding new Tauri setup code.
5. Keep frontend transport generic in `crates/fricon-ui/frontend/src/shared/lib/tauri.ts`, and put feature-specific normalization in `frontend/src/features/<feature>/api/`.
6. Regenerate `bindings.ts` if command or event signatures changed.
7. If the feature needs extra Tauri permissions, update `crates/fricon-ui/capabilities/default.json` and `tauri.conf.json` only as required by the concrete feature.

## Example Triggers

- "Add a new Rust command and call it from the frontend"
- "Expose a typed Tauri event to React"
- "Change tray or window close behavior"
- "Add a plugin permission to the default capability"
- "Update `tauri.conf.json` for a concrete desktop feature"
