# Tauri Security and IPC

Use this reference when a task involves Rust commands, IPC, native APIs, file system access, or security settings.

## IPC Boundary Checklist

- Expose only the minimum commands needed for the feature.
- Validate all command inputs in Rust. Treat frontend data as untrusted.
- Return structured errors; never panic for user-facing operations.
- Prefer explicit command signatures over generic "invoke anything" bridges.
- For this repo, keep Rust as type source-of-truth and export frontend types via `tauri-specta`.

## fricon: tauri-specta Operating Model

- Rust commands and payload types should use `#[specta::specta]` and `specta::Type` where exported.
- Keep unsupported/non-JSON command surfaces (for example binary payload paths) outside first-pass specta adoption.
- Use typed events through `tauri-specta` event support where practical:
  - Define event structs with `Serialize + Deserialize + specta::Type + tauri_specta::Event`.
  - Register events in builder and mount via `Builder::mount_events` during app setup.
  - Consume events from generated bindings in frontend (`events.*`) rather than raw string event names.

## fricon: Binding Generation and CI

- Generate bindings with:
  - `pnpm --filter fricon-ui run gen:bindings`
- Generated file path:
  - `crates/fricon-ui/frontend/src/lib/bindings.ts`
- CI placement:
  - Keep `fmt` fast and static-only.
  - Run bindings generation and `git diff --exit-code` checks in `test` (after Rust build), not in `fmt`.
- Frontend integration:
  - Keep native bridge centralized in `crates/fricon-ui/frontend/src/lib/backend.ts`.
  - Normalize wire types to UI domain types in one place (for example `string -> Date`, strict response guards).

## Command Design

- Keep commands small and specific (one responsibility).
- Use typed payloads and explicit return types.
- Favor async commands for IO to keep UI responsive.
- Prefer data transfer over shared mutable state.

## Frontend Integration

- Centralize native calls in a dedicated module (example: `src/lib/tauri.ts`).
- Wrap calls with typed helpers and normalize errors for the UI.
- Avoid leaking command names throughout the UI.

## Permissions and Allowlist

- Enable only the exact APIs needed for the feature.
- Scope any file system or shell access to user intent and narrow paths.
- Avoid broad permissions that grant access across the whole system.

## File System and Paths

- Always use user-picked paths from dialogs when possible.
- Reject `..` traversal, unexpected absolute paths, or non-file URLs.
- Normalize and validate inputs before use.

## CSP and Webview Safety

- Prefer strict CSP and avoid `unsafe-eval` or broad `unsafe-inline` unless unavoidable.
- Keep external network access explicit and minimal.
- Disable devtools and debug-only features in production builds.

## Diagnostics

- Log command failures with enough context to debug, but avoid logging secrets.
- Surface user-facing errors with actionable messages.
