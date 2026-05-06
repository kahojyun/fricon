# SPEC-001: Validation

## Status

Draft.

## Validation Strategy

Use focused cross-surface tests because the slice crosses Rust core/service,
Python SDK, and Desktop.

## Required Checks

| Area | Validation |
| --- | --- |
| Rust domain/storage | Unit and integration tests for lifecycle transitions, record links, compatibility gates, and recovery state. |
| Dataset artifact | Tests for append, finish, abort, scan schema validation, semantic reads, fixed arrays, and variable-length traces once designed. |
| Service API | Tests for negotiate-before-write, active writer isolation, live event ordering, and mismatch diagnostics. |
| Python SDK | Integration tests for `fricon.library()`, measurement-scoped dataset writes, crash/abort behavior, and reopen snippets. |
| Desktop | Browser/unit tests for measurement console state, active/recent lists, live update rendering, and compatibility diagnostics. |
| Export | Deferred until export format ADR, but keep measurement-centered export acceptance in scope. |
| Docs | Docs-next traceability and public docs updated only after behavior lands. |

## Scenario Checks

1. Run a 1D measurement and watch a live line/scatter view.
2. Run a 2D measurement and watch a live heatmap with explicit axes.
3. Record a measurement without sample context, then attach/correct context.
4. Crash or interrupt a measurement and verify partial data remains readable.
5. Reopen produced datasets from Python using stable IDs.
6. Attempt to write with an incompatible client and verify failure before
   mutation.
