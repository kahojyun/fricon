# Testing Strategy

## Purpose

This note defines the steady-state automated testing strategy for `fricon`.

The goal is to keep the test stack layered, explicit, and maintainable:

- fast tests should stay fast
- UI behavior should be validated in a real browser
- desktop runtime verification should stay small and intentional

## Current Stack

The repo uses:

- Rust tests through `cargo nextest` in CI, with doctests kept separate
- Python tests through `uv run pytest`
- Frontend tests through split Vitest projects:
    - `unit` for fast unit and `jsdom` coverage
    - `browser` for browser-backed UI and integration coverage
- Desktop smoke coverage through a small WebdriverIO + `tauri-driver` suite on
  Windows CI

Relevant files:

- `.github/workflows/ci.yml`
- `.config/nextest.toml`
- `crates/fricon-ui/frontend/vite.config.ts`
- `crates/fricon-ui/frontend/.dependency-cruiser.cjs`
- `crates/fricon-ui/frontend/src/shared/test/setup.ts`
- `tests/desktop-smoke/wdio.conf.mjs`
- `tests/desktop-smoke/specs/app.smoke.test.mjs`
- `crates/fricon-ui/src/bin/create-smoke-workspace.rs`

## Target Split

The intended long-term split is:

```text
pure logic / reducers / data shaping        -> fast Vitest unit tests
hooks and small adapter tests               -> Vitest unit/jsdom tests
React UI behavior and frontend integration  -> Vitest Browser Mode + Playwright
desktop runtime and packaging smoke         -> Tauri WebDriver
Rust workspace tests                        -> cargo nextest
Python API / bindings tests                 -> pytest
```

## Why This Split

### Rust

`cargo nextest` is the preferred CI runner because it is designed for CI
execution, supports CI-specific profiles, and keeps room for future run
partitioning or archiving.

Steady-state CI command:

```sh
cargo nextest run --workspace --profile ci
```

If doctests are required, keep them separate:

```sh
cargo test --workspace --doc
```

### Frontend

Frontend tests should not all live in one bucket.

Use fast unit tests for:

- model reducers
- pure formatting helpers
- API normalization helpers
- low-level rendering math
- hooks that do not need real browser behavior

Use browser-backed tests for:

- routing
- focus and keyboard behavior
- dialogs, menus, and overlays
- query-driven screens
- realistic interaction across multiple UI components

### Desktop

Browser-backed frontend tests do not replace desktop runtime verification.

Desktop smoke coverage should remain intentionally small and verify only the
highest-value paths:

- app launches
- workspace opens
- dataset list renders
- one dataset opens
- one chart view renders

## Placement Rules For New Tests

### Vitest unit or jsdom tests

Default here when the code under test is primarily:

- a pure function
- a reducer
- a feature-local model helper
- an API client adapter with mocked dependencies
- a hook that does not rely on real layout, focus, or browser semantics

### Vitest Browser Mode tests

Default here when the test validates:

- user-visible behavior
- interaction across multiple components
- real browser event handling
- focus management
- layout-sensitive behavior
- a feature screen that is already close to integration scope

### Tauri WebDriver smoke tests

Use only for:

- packaged app launch checks
- desktop runtime integration
- platform or WebView regressions

Do not use Tauri WebDriver for broad component coverage.

## Frontend File Conventions

Recommended conventions:

- `*.test.*` for default unit and `jsdom` tests
- `*.browser.test.*` for Vitest Browser Mode tests
- `*.smoke.test.*` for Tauri desktop smoke tests
- `test-utils.*` for colocated test helpers

If a helper is shared only by browser-mode tests, prefer placing it under a
browser-specific test directory instead of adding a runner prefix to the helper
filename.

For `src/app/**`, `src/features/**/ui/**`, and
`src/features/**/rendering/**`, UI-facing render tests should default to
`*.browser.test.*`.

The frontend `dependency-cruiser` config blocks new
`@testing-library/react` and `@testing-library/user-event` imports in default
`*.test.*` files in those directories. Unit-scoped tests that need those
libraries should live under `hooks/`, `api/`, or `model/`, not under `ui/`.

Do not encode extra scope labels such as `integration` in the filename unless
they are required by a tool. Test scope should usually live in the directory
and in the test description, while the filename suffix should communicate the
execution mode.

Vitest should stay split into explicit projects:

```text
test.projects
  - unit
  - browser
```

## Frontend Command Policy

Keep frontend test commands explicit.

- `pnpm run check` from repo root is the default frontend quality gate
- `pnpm run test` from repo root runs the full frontend batch
- `pnpm run test:unit` from repo root runs only unit/jsdom tests
- `pnpm run test:browser` from repo root runs only browser-mode tests
- `pnpm run test:browser:headed` from repo root is the opt-in headed debug
  variant
- `pnpm run test:smoke` from repo root runs the desktop smoke suite; the
  supported CI path is Windows-only

Do not rely on the batch `test` script for file filters, watch mode, or
project selection. When a targeted rerun is needed, choose the test mode first
and pass the filter to `test:unit` or `test:browser`.

## Intentional Unit Exceptions

Most UI-facing coverage should already be browser-backed. The remaining
unit-scoped frontend tests should be intentional.

Examples that should stay in fast unit land:

- `crates/fricon-ui/frontend/src/shared/lib/tauri.test.ts`
- `crates/fricon-ui/frontend/src/features/workspace/api/client.test.ts`
- `crates/fricon-ui/frontend/src/features/datasets/api/*.test.ts`
- `crates/fricon-ui/frontend/src/features/datasets/model/*.test.ts`
- `crates/fricon-ui/frontend/src/features/datasets/hooks/*.test.ts*`
- `crates/fricon-ui/frontend/src/features/charts/model/*.test.ts`
- `crates/fricon-ui/frontend/src/features/charts/rendering/webgl.test.ts`
- `crates/fricon-ui/frontend/src/features/charts/rendering/rendererBounds.test.ts`
- `crates/fricon-ui/frontend/src/features/charts/rendering/numericLabelFormat.test.ts`
- `crates/fricon-ui/frontend/src/features/charts/rendering/d3Overlay.test.ts`
- `crates/fricon-ui/frontend/src/features/charts/rendering/crosshairOverlay.test.ts`
- `crates/fricon-ui/frontend/src/features/charts/rendering/zoomController.test.ts`
- `crates/fricon-ui/frontend/src/features/charts/hooks/useWebGLChart.test.tsx`

These tests are closer to pure logic, synthetic adapter behavior, or
low-level rendering math and should not be migrated just for consistency.

## Shared Test Setup Debt

Only one shared browser API shim remains in the unit setup:

- `ResizeObserver` in `crates/fricon-ui/frontend/src/shared/test/setup.ts`

It remains because `useWebGLChart.test.tsx` still mounts a hook that subscribes
to `ResizeObserver` under `jsdom`. If that test is ever redesigned or moved to
browser mode, this shim should be reconsidered.

## Guardrails

- Do not move tests between runners mechanically; choose the runner based on
  what behavior the test is actually validating.
- Do not force low-level rendering math, reducer logic, or synthetic adapter
  tests into browser mode just for consistency.
- Do not use desktop smoke tests as a general frontend integration layer; keep
  them limited to runtime and packaging validation.
- Do not place new `jsdom`-heavy UI interaction tests under `ui/`; move the
  test to browser mode or move the logic under test to `hooks/`, `api/`, or
  `model/`.
- Do not add new path-based exceptions when moving a file to `hooks/`, `api/`,
  or `model/` would clarify ownership instead.
