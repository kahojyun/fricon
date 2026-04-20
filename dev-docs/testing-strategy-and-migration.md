# Testing Strategy And Migration Plan

## Purpose

This note defines the target automated testing stack for `fricon`, explains why
the current frontend test setup is transitional rather than the desired steady
state, and lays out a multi-PR migration plan that can be executed while
keeping CI green.

The goal is to reduce long-term test debt, not just to add another test runner.

## Current State

Today the repo uses:

- Rust tests through `cargo test` in CI.
- Python tests through `uv run pytest`.
- Frontend tests through a single `vitest` command backed by `jsdom`.

Relevant current files:

- `.github/workflows/ci.yml`
- `crates/fricon-ui/frontend/vite.config.ts`
- `crates/fricon-ui/frontend/src/shared/test/setup.ts`

Current frontend constraints:

- The Vitest config is explicitly `jsdom`-based.
- The shared setup file contains browser API shims such as `matchMedia` and
  `ResizeObserver`.
- Many React UI tests are integration-style tests but still run in a simulated
  DOM.

That setup is useful for fast iteration, but it should be treated as
transitional for UI-heavy tests.

## Target State

The steady-state testing stack should be layered:

- Rust: `cargo nextest` for CI test execution.
- Python: `uv run pytest`.
- Frontend unit and logic tests: Vitest in `node` or `jsdom`.
- Frontend UI and integration tests: Vitest Browser Mode with Playwright
  provider.
- Desktop smoke tests: a very small Tauri WebDriver suite on Linux and Windows.

This is the intended long-term split:

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

`cargo-nextest` is a better CI runner than `cargo test` for this repo because
it is designed for CI execution, supports CI-specific profiles, and makes it
easier to partition or archive runs later if needed.

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
- realistic user interaction across multiple UI components

### Desktop

Browser-backed frontend tests do not replace desktop runtime verification.

Tauri smoke coverage should remain intentionally small and verify only the
highest-value end-to-end paths:

- app launches
- workspace opens
- dataset list renders
- one dataset opens
- one chart view renders

## Testing Rules For New Code

Use these placement rules for new tests.

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

## Frontend Steady-State File Conventions

Recommended conventions:

- `*.test.ts` for pure unit tests.
- `*.test.tsx` for small component or hook tests that remain in unit/jsdom
  scope.
- `*.browser.test.tsx` for Vitest Browser Mode tests.
- `*.smoke.ts` or a dedicated smoke directory for Tauri desktop smoke tests.

Vitest should be split into explicit projects instead of one global mode.

Recommended high-level shape:

```text
test.projects
  - unit
  - browser
```

## Frontend Command Policy

Keep frontend test commands explicit.

- `pnpm run test` from repo root runs the full frontend batch.
- `pnpm run test:unit` from repo root runs only unit/jsdom tests.
- `pnpm run test:browser` from repo root runs only browser-mode tests.
- `pnpm run test:browser:headed` from repo root is the opt-in headed debug
  variant.

Do not rely on the batch `test` script for file filters, watch mode, or
project selection. When a targeted rerun is needed, choose the test mode first
and pass the filter to `test:unit` or `test:browser`.

## Current Frontend Test Buckets

The current frontend suite is roughly split between `.ts` logic tests and
`.tsx` UI tests. The following buckets should guide migration work.

### Keep in fast unit land

Examples:

- `crates/fricon-ui/frontend/src/shared/lib/tauri.test.ts`
- `crates/fricon-ui/frontend/src/features/workspace/api/client.test.ts`
- `crates/fricon-ui/frontend/src/features/datasets/api/*.test.ts`
- `crates/fricon-ui/frontend/src/features/datasets/model/*.test.ts`
- `crates/fricon-ui/frontend/src/features/charts/model/*.test.ts`
- `crates/fricon-ui/frontend/src/features/charts/rendering/webgl.test.ts`
- `crates/fricon-ui/frontend/src/features/charts/rendering/rendererBounds.test.ts`
- `crates/fricon-ui/frontend/src/features/charts/rendering/numericLabelFormat.test.ts`
- `crates/fricon-ui/frontend/src/features/charts/rendering/d3Overlay.test.ts`
- `crates/fricon-ui/frontend/src/features/charts/rendering/crosshairOverlay.test.ts`
- `crates/fricon-ui/frontend/src/features/charts/rendering/zoomController.test.ts`

These tests are already closer to unit or synthetic adapter tests and should
not be migrated just for consistency.

### Migrate to browser-backed frontend tests

First-wave candidates:

- `crates/fricon-ui/frontend/src/app/router.test.tsx`
- `crates/fricon-ui/frontend/src/app/ui/DatasetExplorerScreen.integration.test.tsx`
- `crates/fricon-ui/frontend/src/app/ui/DatasetInspector.test.tsx`
- `crates/fricon-ui/frontend/src/features/datasets/ui/DatasetTable.test.tsx`
- `crates/fricon-ui/frontend/src/features/datasets/ui/DatasetTableToolbar.test.tsx`
- `crates/fricon-ui/frontend/src/features/datasets/ui/DatasetTagFilter.test.tsx`
- `crates/fricon-ui/frontend/src/features/datasets/ui/DatasetTableRowActions.test.tsx`
- `crates/fricon-ui/frontend/src/features/datasets/ui/ManageTagsDialog.test.tsx`
- `crates/fricon-ui/frontend/src/features/datasets/ui/DatasetPropertiesPanel.test.tsx`
- `crates/fricon-ui/frontend/src/features/charts/ui/ChartViewer.test.tsx`
- `crates/fricon-ui/frontend/src/features/charts/ui/FilterTable.test.tsx`

These tests are already user-interaction heavy and are the best candidates for
retiring `jsdom` debt.

### Migrate later or redesign

Higher-risk files:

- `crates/fricon-ui/frontend/src/features/charts/hooks/useWebGLChart.test.tsx`
- `crates/fricon-ui/frontend/src/features/charts/ui/ChartWrapper.test.tsx`

These tests currently stub WebGL and browser primitives heavily. They should be
reviewed deliberately instead of being ported mechanically.

## Migration Plan

The migration should be executed in multiple PRs so that each change set is
reviewable and reversible.

### PR1: CI runner foundations

Changes:

- Add `.config/nextest.toml` with a `ci` profile.
- Replace Rust CI execution from `cargo test` to `cargo nextest`.
- Keep doctests separate if needed.
- Add explicit frontend scripts for `test`, `test:unit`, and `test:browser`.
- Make the repo-root scripts the canonical frontend test entrypoints.
- Split Vitest configuration into `unit` and `browser` projects.
- Add Playwright-backed browser provider dependencies and CI install step.

Acceptance criteria:

- CI stays green with existing frontend tests still running in unit mode.
- Browser project exists and can run at least a trivial test in CI.
- Rust tests pass under `cargo nextest`.

Risk:

- Low

### PR2: Browser harness and conventions

Changes:

- Add browser-specific setup utilities for React Query, router, and mocked
  Tauri runtime state.
- Establish the `*.browser.test.tsx` naming convention.
- Validate the harness with an intentionally small migrated browser test.

Acceptance criteria:

- A minimal browser-mode test runs headless in CI.
- Developers have a clear pattern to copy.

Risk:

- Low

### PR3: App shell and routing migration

Changes:

- Port `src/app/router.test.tsx` to browser mode.
- Port `src/app/ui/DatasetInspector.test.tsx` if it fits the new harness cleanly.
- Delete or shrink the replaced `jsdom` coverage once parity is confirmed.

Acceptance criteria:

- Route navigation and shell behavior are validated in a real browser.
- CI demonstrates stable browser test execution.

Risk:

- Low to medium

### PR4: Dataset UI migration

Changes:

- Port dataset UI behavior tests to browser mode.
- Start with:
    - `DatasetTable.test.tsx`
    - `DatasetTableToolbar.test.tsx`
    - `DatasetTagFilter.test.tsx`
    - `DatasetTableRowActions.test.tsx`
    - `ManageTagsDialog.test.tsx`
    - `DatasetPropertiesPanel.test.tsx`
- Delete old `jsdom` copies after parity is proven.

Acceptance criteria:

- Dataset UI behavior is browser-backed rather than `jsdom`-simulated.
- Unit tests remain only where they add speed or isolation value.

Risk:

- Medium

### PR5: Frontend integration slice migration

Changes:

- Port `DatasetExplorerScreen.integration.test.tsx` to browser mode.
- Simplify mocks where they exist only to compensate for `jsdom`.

Acceptance criteria:

- At least one high-value screen flow is covered in browser mode end to end
  within the frontend.

Risk:

- Medium to high

### PR6: Chart UI migration

Changes:

- Port UI-facing chart tests first:
    - `ChartViewer.test.tsx`
    - `FilterTable.test.tsx`
    - `ChartLegend.test.tsx`
    - `ChartTooltip.test.tsx`
- Re-evaluate whether low-level WebGL hook tests should stay synthetic or be
  replaced by a smaller number of browser-level assertions.

Acceptance criteria:

- Chart UI interactions run in a real browser.
- Low-level WebGL tests have an intentional long-term home.

Risk:

- High

### PR7: Desktop smoke coverage

Changes:

- Add a very small Tauri WebDriver suite for Linux and Windows.
- Keep it limited to runtime/package smoke validation.

Acceptance criteria:

- CI can catch app-launch and desktop-runtime regressions separately from
  frontend component failures.

Risk:

- Medium to high

### PR8: Cleanup and policy lock-in

Changes:

- Remove obsolete `jsdom` shims from `src/shared/test/setup.ts` where they are
  no longer needed.
- Document final placement rules in dev docs and contributor guidance.
- Add guardrails so new UI integration tests do not silently fall back to
  `jsdom`.

Acceptance criteria:

- The new testing boundaries are explicit and durable.
- The repo does not drift back toward `jsdom`-heavy UI integration coverage.

Risk:

- Low

## Ordering

Recommended PR order:

1. PR1: CI runner foundations
2. PR2: Browser harness and conventions
3. PR3: App shell and routing migration
4. PR4: Dataset UI migration
5. PR5: Frontend integration slice migration
6. PR6: Chart UI migration
7. PR7: Desktop smoke coverage
8. PR8: Cleanup and policy lock-in

PR4 and PR5 may overlap once the browser harness is stable.

## What Not To Do

- Do not migrate all `.tsx` tests blindly.
- Do not force low-level rendering math tests into browser mode.
- Do not adopt Playwright Component Testing as the primary migration path
  unless the frontend test style is intentionally rewritten around its
  Node/browser boundary.
- Do not expand desktop smoke tests into a large end-to-end suite before
  browser-backed frontend coverage is stable.

## Definition Of Done

The migration is complete when:

- Rust CI uses `cargo nextest`.
- New UI integration tests default to Vitest Browser Mode.
- Most existing `jsdom` UI/integration tests have been retired.
- Remaining `jsdom` tests are intentionally unit-scoped.
- A very small Tauri desktop smoke suite covers runtime/package regressions.

At that point the repo will have a layered testing strategy instead of a
collection of historically accumulated runner choices.
