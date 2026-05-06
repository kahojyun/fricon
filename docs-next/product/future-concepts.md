# Future Concepts

## Status

Draft future ledger.

## Purpose

Preserve important v0.3+ and v0.4+ direction without letting future systems
inflate the first v0.2 measurement slice.

## Promotion Rule

A future concept can move into implementation only after it has a clear story,
domain owner, ADR if needed, and does not make the core measurement loop harder
to use.

## Candidate v0.3 Product Assistance

- FC-001: Read-only LAN monitoring for viewing, browsing, and export without
  remote writes.
- FC-002: Rich sample fields, 2D sample maps, and saved views.
- FC-003: Measurement-code source setup that replaces copied-code folders with
  approved-code update flows, local checkout/environment guidance, and no
  central Fricon server.

## Candidate v0.4 Automation Foundation

- FC-004: Parameter profiles and proposals.
- FC-005: Analysis records that consume artifacts and produce derived outputs.
- FC-006: Calibration records with reviewable proposals.
- FC-007: Managed code snapshots and execution.
- FC-008: Device identity and managed communication.
- FC-009: Managed measurement plans.

## Later Or ADR-Gated

- FC-010: AI-assisted automation after the audit model exists.
- FC-011: Resumable execution checkpoints with a managed runner.
- FC-012: External large asset references for detector files, images, and
  waveforms.
- FC-013: User-facing dataset streams, if internal stream support proves useful
  enough to expose later.

## Historical Inputs

Use `dev-docs/v0.2/future-concepts.md` and files under
`dev-docs/v0.2/archive/` as source material only. If they conflict with
accepted `docs-next/` documents, the accepted `docs-next/` document wins.
