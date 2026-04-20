# Testing Strategy And Migration Plan

## Purpose

This note defines the target automated testing stack for `fricon`, explains why
the current frontend test setup is transitional rather than the desired steady
state, and lays out a multi-PR migration plan that can be executed while
keeping CI green.

The goal is to reduce long-term test debt, not just to add another test runner.

## Current State

The repo now uses:

- Rust tests through `cargo nextest` in CI, with doctests kept separate.
- Python tests through `uv run pytest`.
- Frontend tests through split Vitest projects:
    - `unit` for fast unit and `jsdom` coverage
    - `browser` for browser-backed UI and integration coverage
- Desktop smoke coverage through a tiny WebdriverIO + `tauri-driver` suite on
  Windows CI.

Relevant current files:

- `.github/workflows/ci.yml`
- `.config/nextest.toml`
- `crates/fricon-ui/frontend/vite.config.ts`
- `crates/fricon-ui/frontend/src/shared/test/setup.ts`
- `crates/fricon-ui/frontend/src/shared/test/browser/`
- `tests/desktop-smoke/wdio.conf.mjs`
- `tests/desktop-smoke/specs/app.smoke.test.mjs`
- `crates/fricon-ui/src/bin/create-smoke-workspace.rs`

Current frontend constraints:

- The browser-mode migration is underway, not finished.
- Only one shared browser API shim remains in the unit setup:
    - `ResizeObserver`, kept for the intentionally retained `useWebGLChart` /
      `ChartWrapper` unit coverage
- A few higher-risk chart and WebGL-adjacent tests are still intentionally left
  in unit land.

This means the repo is already on the target path, but the testing boundaries
are not fully finalized yet.

## Progress Snapshot

As of PR [#434](https://github.com/kahojyun/fricon/pull/434), the migration has
already covered:

- the CI and runner foundation work
- the browser test harness and naming convention
- the first browser migration wave for app shell, dataset UI, and one
  frontend integration slice
- most of the low-risk chart UI migration
- command-surface cleanup plus matching updates to docs, `AGENTS.md`, and
  skills
- the desktop smoke phase with a deterministic workspace fixture, a single
  WebdriverIO smoke spec, and a dedicated Windows CI job

In terms of the original plan, this PR effectively collapses the old PR1
through PR5 and most of PR6 into one reviewed unit.

This roadmap should therefore be read as delivery phases, not as a promise
that each phase maps one-to-one to a pull request.

## Target State

The steady-state testing stack should be layered:

- Rust: `cargo nextest` for CI test execution.
- Python: `uv run pytest`.
- Frontend unit and logic tests: Vitest in `node` or `jsdom`.
- Frontend UI and integration tests: Vitest Browser Mode with Playwright
  provider.
- Desktop smoke tests: a very small Tauri WebDriver suite on Windows.

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

- `*.test.*` for default unit and `jsdom` tests.
- `*.browser.test.*` for Vitest Browser Mode tests.
- `*.smoke.test.*` for Tauri desktop smoke tests.
- `test-utils.*` for colocated test helpers.

If a helper is shared only by browser-mode tests, prefer placing it under a
browser-specific test directory instead of adding a runner prefix to the helper
filename.

For `src/app/**`, `src/features/**/ui/**`, and
`src/features/**/rendering/**`, UI-facing render tests should default to
`*.browser.test.*`. The frontend `dependency-cruiser` config now blocks new
`@testing-library/react` and `@testing-library/user-event` imports in default
`*.test.*` files in those directories. The remaining carve-outs should stay
small and principled: explicit chart-unit holdouts plus `use*.test.*` hook
tests that remain intentionally unit-scoped even when they live under `ui/`.

Do not encode extra scope labels such as `integration` in the filename unless
they are required by a tool. Test scope should usually live in the directory
and in the test description, while the filename suffix should communicate the
execution mode.

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
- `pnpm run test:smoke` from repo root runs the desktop smoke suite.
  The supported CI path is Windows-only.

Do not rely on the batch `test` script for file filters, watch mode, or
project selection. When a targeted rerun is needed, choose the test mode first
and pass the filter to `test:unit` or `test:browser`.

## Current Frontend Test Buckets

The current frontend suite contains a mix of logic tests and UI tests. The
following buckets should guide migration work.

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
- `crates/fricon-ui/frontend/src/app/ui/DatasetExplorerScreen.browser.test.tsx`
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

The migration should still be executed in multiple PRs, but the original
eight-PR ladder was more granular than the implementation reality justified.
The work fell into a smaller number of tightly coupled phases, and future PRs
may cover one phase, multiple phases, or part of a phase depending on review
scope.

### Phase 1: Foundation

Changes:

- Add `.config/nextest.toml` with a `ci` profile.
- Replace Rust CI execution from `cargo test` to `cargo nextest`.
- Keep doctests separate.
- Split Vitest configuration into `unit` and `browser` projects.
- Add Playwright-backed browser dependencies and CI install step.
- Establish the canonical repo-root frontend test commands.
- Add browser harness utilities and the `*.browser.test.*` convention.
- Update docs, `AGENTS.md`, and skills to match the new command surface.

Acceptance criteria:

- CI stays green.
- Rust tests pass under `cargo nextest`.
- Browser-mode tests can run headless in CI.
- Developers have one clear frontend command surface to follow.

Risk:

- Low to medium

Status:

- Covered by PR #434.

### Phase 2: First browser migration wave

Changes:

- Port the clearly safe UI and integration tests to browser mode:
    - app shell and routing
    - dataset inspector
    - dataset UI behavior tests
    - one high-value frontend integration screen flow
- Delete old `jsdom` copies after parity is proven.

Acceptance criteria:

- Route and dataset UI behavior are browser-backed rather than
  `jsdom`-simulated.
- At least one high-value screen flow is covered end to end within the
  frontend.

Risk:

- Medium

Status:

- Covered by PR #434.

### Phase 3: Chart UI migration

Changes:

- Port the low-risk chart UI tests:
    - `ChartViewer.test.tsx`
    - `FilterTable.test.tsx`
    - `ChartLegend.test.tsx`
    - `ChartTooltip.test.tsx`
- Explicitly leave low-level WebGL-heavy tests in unit land unless a redesign
  justifies a smaller browser-backed replacement.

Acceptance criteria:

- Chart UI interactions run in a real browser.
- Low-level chart tests have an intentional long-term home.

Risk:

- Medium to high

Status:

- Mostly covered by PR #434, but follow-up cleanup or reclassification may
  still be needed.

### Phase 4: Desktop smoke coverage

Changes:

- Add a very small Tauri WebDriver suite for Windows.
- Keep it limited to runtime/package smoke validation.

Acceptance criteria:

- CI can catch app-launch and desktop-runtime regressions separately from
  frontend component failures.

Risk:

- Medium to high

Status:

- Covered by the current implementation:
    - deterministic workspace fixture generation
    - one minimal app-launch / dataset-open / chart-render smoke path
    - separate Windows CI job

### Phase 5: Cleanup and policy lock-in

Changes:

- Remove obsolete `jsdom` shims from `src/shared/test/setup.ts` where they are
  no longer needed.
- Document the final placement rules in dev docs and contributor guidance.
- Add guardrails so new UI integration tests do not silently fall back to
  `jsdom`.

Acceptance criteria:

- The new testing boundaries are explicit and durable.
- The repo does not drift back toward `jsdom`-heavy UI integration coverage.

Risk:

- Low

Status:

- In progress:
    - obsolete `matchMedia` and `getAnimations` unit shims removed
    - only `ResizeObserver` remains as a shared unit shim
    - `dependency-cruiser` guardrails now block new UI-facing render tests from
      defaulting to `*.test.*` in app/ui/rendering directories

## Ordering

Recommended remaining order:

1. Phase 5: Cleanup and policy lock-in

If chart-specific follow-up is still needed after review, treat it as a small
continuation of Phase 3 rather than reopening the old eight-step plan.

## Expected Remaining PR Count

Based on what has already landed and on the coupling we observed during the
first implementation wave, the remaining migration work will likely take:

- 1 PR in the most likely case:
    - one PR for cleanup, guardrails, and final policy lock-in
- 2 PRs if chart-specific follow-up needs to be split out for review clarity
  before or alongside the cleanup work

The most realistic planning assumption is therefore that the migration can be
finished in 1 to 2 additional PRs after the desktop smoke phase lands.

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
