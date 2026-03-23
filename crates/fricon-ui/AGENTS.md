## Crate-Specific Constraints

- Keep top-level boundaries small and explicit: `desktop_runtime/` owns app/runtime/session lifecycle, `tauri_api.rs` owns command/event registration and shared API errors, and `features/` owns user-facing behavior.
- `features/<feature>/tauri.rs` is the only required feature entrypoint. It owns Tauri commands/events, Specta-exported types, native dialogs, and mapping feature errors to API errors.
- Below `tauri.rs`, organize modules by behavior or use case. `workflow.rs` is optional and should exist only when it groups real orchestration.
- Non-`tauri.rs` feature modules must not depend on Tauri or `rfd`.
- Prefer using `fricon` types directly inside feature logic. Add feature-local types or conversions only when the frontend shape differs or a UI-specific type has real meaning.
- Keep shared session state and caches under runtime ownership, not under a feature.
- Avoid traits, shared helpers, or wrapper modules introduced only for pattern conformity.
- After Rust Tauri command/event signature changes, regenerate bindings with `pnpm --filter fricon-ui run gen:bindings`.
- Update Tauri capabilities or config only when the concrete feature requires it.
