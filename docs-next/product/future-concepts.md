# Future Concepts

## Status

Accepted future ledger.

## Purpose

Preserve important v0.3+ and v0.4+ direction without letting future systems
inflate the first v0.2 measurement slice.

## Promotion Rule

A future concept can move into implementation only after it has a clear story,
domain owner, ADR if needed, and does not make the core measurement loop harder
to use.

## Candidate v0.3 Product Assistance

- FC-004: Parameter profiles, immutable snapshot binding, history, proposals,
  and diff views. This is the first v0.3+ product direction because large
  parameter sets are already a standalone user pain.
- FC-003: Measurement-code source setup that replaces copied-code folders with
  approved-code update flows, local checkout/environment guidance, and no
  central Fricon server.
- FC-007: Managed code snapshots and opt-in managed run capture. Design early
  because useful run history depends on automatic capture, not manual entry.
- FC-014: Measurement history, compare, and operator handoff generated from
  captured parameter, code, setup, lifecycle, and artifact facts.
- FC-001: Read-only LAN monitoring for viewing, browsing, and export without
  remote writes.
- FC-002: Rich sample fields, 2D sample maps, and saved views.

## Candidate v0.4 Automation Foundation

- FC-005: Analysis records that consume artifacts and produce derived outputs.
- FC-006: Calibration records with reviewable proposals.
- FC-008: Device identity and managed communication.
- FC-009: Managed measurement plans.
- FC-015: Workflow preview layer for calibration, benchmark, and reviewed
  automation flows.

## Later Or ADR-Gated

- FC-010: AI-assisted automation after the audit model exists.
- FC-011: Resumable execution checkpoints with a managed runner.
- FC-012: External large asset references for detector files, images, and
  waveforms.
- FC-013: User-facing dataset streams, if internal stream support proves useful
  enough to expose later.
- FC-016: Applying stored parameter or setup state back to devices. Store and
  diff comes first; write-back needs safety, partial-failure, readback, and
  audit ADRs.

## Interview Direction

Current v0.3+ product bias:

- Parameter system first.
- Managed code source and managed run design early.
- Useful run history is derived from captured facts rather than user-entered
  history.
- Lab state starts with store-and-diff, not device apply.
- Parameter model starts as a hybrid tree: flexible structured snapshots first,
  with selected parameters upgraded to typed definitions, units, validation,
  and UI affordances.
- Managed runner minimum is SDK runner integration, not a blind shell-command
  wrapper and not a scheduler.
- Parameter profile changes use light proposals with actor, reason, and
  approval history, without requiring a full permissions system.

Candidate future stories and requirements live in
`product/future-stories-and-requirements.md`.

## Historical Inputs

Use `dev-docs/v0.2/future-concepts.md` and files under
`dev-docs/v0.2/archive/` as source material only. If they conflict with
accepted `docs-next/` documents, the accepted `docs-next/` document wins.
