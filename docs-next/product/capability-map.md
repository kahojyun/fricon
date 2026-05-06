# Capability Map

## Status

Draft.

## ID Rules

Capability IDs are stable. Do not reuse or renumber them.

Keep this file compact. Put detailed acceptance notes in epics, user stories,
or specs instead of expanding this map into a large table.

## v0.2 Foundation Capabilities

- CAP-001: Local data library.
- CAP-002: Install and launch local Fricon.
- CAP-003: Python measurement recording.
- CAP-004: Optional, visible, correctable sample/session context.
- CAP-005: Dataset artifact recording.
- CAP-006: Dataset scan semantics.
- CAP-007: Nonblocking live inspection.
- CAP-008: Lifecycle and readable partial recovery.
- CAP-009: Notes, markers, favorites, pins, and optional tags.
- CAP-010: Python reopen through stable IDs and public APIs.
- CAP-011: Measurement-centered export.
- CAP-012: Backup, restore, and migration checkpoints.
- CAP-013: Compatibility diagnostics and fail-before-write checks.
- CAP-014: Honest code provenance summary.
- CAP-015: Flexible parameter snapshot without a registry UI.
- CAP-016: Light measurement attachments.
- CAP-017: Lightweight operator profile and audit actor.
- CAP-026: Dataset artifact discovery and direct open.
- CAP-027: Passive setup and environment summary.
- CAP-028: Scan schema authoring helpers plus raw schema escape hatch.
- CAP-029: Passive procedure summary.
- CAP-030: Migration ergonomics for Data Vault-style new measurement scripts.

## Later Capabilities

- CAP-018: Read-only remote monitoring.
- CAP-019: Rich sample maps and saved views.
- CAP-020: Measurement-code source setup and approved code update flows.
- CAP-021: Parameter profiles and proposals.
- CAP-022: Analysis and calibration records.
- CAP-023: Managed code snapshots and execution.
- CAP-024: Device boundary and managed device communication.
- CAP-025: AI-assisted reviewed automation.

## Product Grouping

For planning, group capabilities by user outcome:

- Local adoption: CAP-001, CAP-002, CAP-012, CAP-013.
- New measurement replacement: CAP-003, CAP-005, CAP-006, CAP-007, CAP-028,
  CAP-030.
- Context and provenance: CAP-004, CAP-014, CAP-015, CAP-017, CAP-027,
  CAP-029.
- Review and analysis: CAP-008, CAP-009, CAP-010, CAP-011, CAP-016, CAP-026.
- Future automation foundation: CAP-018 through CAP-025.
