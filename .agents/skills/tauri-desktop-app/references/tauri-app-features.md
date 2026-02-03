# Tauri App Features

Use this reference for windowing, menus, tray, deep links, updater, and packaging tasks.

## Windowing and Lifecycle

- Decide which actions need separate windows vs in-app panels.
- Persist window size, position, and last state when appropriate.
- Ensure close/minimize behavior matches platform expectations.

## Menus, Tray, and Shortcuts

- Define menu and tray actions as explicit commands.
- Keep shortcuts discoverable and avoid collisions with OS defaults.
- Consider background behavior when the window is hidden but app should remain active.

## Deep Links and Protocols

- Validate deep link payloads before use.
- Route deep link actions through the same command layer as normal UI flows.

## Auto-Updater and Releases

- Decide the update strategy early (silent vs user-confirmed).
- Make update channels explicit (stable, beta).
- Keep release signing and verification steps documented.

## Packaging and Production

- Confirm platform targets and required signing identities.
- Validate that production builds disable dev-only features.
- Audit bundle size and startup time; avoid heavy preload scripts.

## Config Touchpoints

- Track changes to Tauri config alongside feature changes.
- Keep any allowlist or permissions aligned with the features you ship.
- Document any required environment variables or build flags.
