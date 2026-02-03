# Tauri Security and IPC

Use this reference when a task involves Rust commands, IPC, native APIs, file system access, or security settings.

## IPC Boundary Checklist

- Expose only the minimum commands needed for the feature.
- Validate all command inputs in Rust. Treat frontend data as untrusted.
- Return structured errors; never panic for user-facing operations.
- Prefer explicit command signatures over generic "invoke anything" bridges.

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
