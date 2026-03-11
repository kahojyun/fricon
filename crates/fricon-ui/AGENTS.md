## Crate-Specific Constraints

- Keep the Rust-side dependency flow one-way: `desktop_runtime -> api::<feature> -> application::<feature> -> fricon`.
- `api/` owns Tauri commands, events, exported DTOs, and binding export.
- `application/` owns feature orchestration and should not depend on Tauri types.
- `desktop_runtime/` owns app lifecycle, tray/window behavior, workspace session ownership, and event forwarding.
- After Rust Tauri command/event signature changes, regenerate bindings with `pnpm --filter fricon-ui run gen:bindings`.
- Update Tauri capabilities or config only when the concrete feature requires it.
