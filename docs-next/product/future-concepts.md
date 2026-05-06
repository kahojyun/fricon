# Future Concepts

## Status

Draft future ledger.

## Purpose

Preserve important v0.3+ and v0.4+ direction without letting future systems
inflate the first v0.2 measurement slice.

## Promotion Rule

A future concept can move into v0.2 implementation only after:

- it has a clear user story and capability mapping
- it has a domain owner
- it has an ADR if it changes storage, service API, compatibility, or mutation
  audit behavior
- it does not make the core measurement loop harder to use

## Concepts

| ID | Concept | Current Status | Notes |
| --- | --- | --- | --- |
| FC-001 | Read-only LAN monitoring | Candidate v0.3 | Remote viewing, browsing, and export only; no remote writes. |
| FC-002 | Rich sample fields and 2D sample maps | Candidate v0.3 | Useful secondary view and filter entry after minimal sample/session support. |
| FC-003 | Measurement code source setup | Candidate v0.3+ | Help install/update approved code without becoming a Git host. |
| FC-004 | Parameter profiles and proposals | Candidate v0.4 | Requires measurement records, parameter snapshots, and audit model first. |
| FC-005 | Analysis records | Candidate v0.3+ | Consume artifacts and produce derived artifacts/results without mutating facts. |
| FC-006 | Calibration records | Candidate v0.4 | Reviewable proposals and accepted/rejected parameter changes. |
| FC-007 | Managed code snapshots and execution | Candidate v0.4 | Requires code source, snapshot, ScriptRun, and resource boundary ADRs. |
| FC-008 | Device identity and managed communication | Candidate v0.4+ | Reserve boundary, but do not build broad framework in v0.2. |
| FC-009 | Managed measurement plans | Candidate v0.4+ | Optional for repeated/automation-heavy work; imperative Python remains valid. |
| FC-010 | AI-assisted automation | Candidate after audit model | Read/suggest first; mutating actions require explicit review and audit. |
| FC-011 | Resumable execution checkpoints | Candidate with managed runner | Requires scan-point checkpoint semantics; v0.2 only promises readable partial data. |
| FC-012 | External large asset references | ADR-gated | Reserve storage/export hooks for detector files, images, and waveforms, but keep scans/traces first. |
| FC-013 | User-facing dataset streams | Candidate later | v0.2 may allow internal streams, but should not expose streams as a normal user concept. |

## Historical Inputs

Use `dev-docs/v0.2/future-concepts.md` and files under
`dev-docs/v0.2/archive/` as source material only. If they conflict with
accepted `docs-next/` documents, the accepted `docs-next/` document wins.
