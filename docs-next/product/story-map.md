# Story Map

## Status

Draft.

## Purpose

Keep the product route readable for future AI sessions. This file is a compact
map, not the place for full acceptance details.

## Backbone

```text
Install and set up
  -> create/open data library
  -> select optional sample/session context
  -> run Python measurement
  -> record dataset artifacts
  -> inspect live
  -> finish/recover
  -> annotate
  -> reopen in Python
  -> export
```

## v0.2 Product Epics

- EPIC-001: Local setup and data-library adoption.
- EPIC-002: New measurement logging replacement.
- EPIC-003: Measurement console and inspection.
- EPIC-004: Recovery, annotation, reopen, and export.
- EPIC-005: Migration ergonomics and future lab scaling.

Detailed epic notes live under `product/epics/`.

## v0.2 Story Index

Foundation:

- US-001: Install and launch Fricon on a lab computer.
- US-002: Create or open one local data library.
- US-010: Update without corrupting measurement work.

Measurement loop:

- US-003: Run an exploratory measurement from Python.
- US-004: Produce dataset artifacts from a measurement.
- US-005: Watch and inspect live data.
- US-015: Declare common scan shapes quickly.

Context and provenance:

- US-007: Annotate at the right level.
- US-011: Record code provenance honestly.
- US-012: Register a sample and sample session when useful.
- US-014: Record passive setup context.
- US-016: Record passive procedure context.

Recovery and analysis:

- US-006: Recover a partial measurement.
- US-008: Reopen measurement outputs from Python.
- US-009: Export a measurement for offline analysis.
- US-013: Find and open a dataset artifact directly.

Migration:

- US-017: Migrate a Data Vault-style script to Fricon writers.
- US-018: Start new work in Fricon while old history stays in the old system.

Expanded story files live under `product/user-stories/` only when the story is
important enough to guide product or implementation decisions.

## Sequencing

M1 should prove enough of US-001 through US-008 to record, inspect, recover, and
reopen a measurement while preserving first-class dataset discovery.

SPEC-002 owns the export/offline-analysis details. Export remains a v0.2
product promise; it is split out only to keep SPEC-001 focused.
