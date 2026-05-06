# EPIC-004: Recovery, Reopen, And Export

## Status

Accepted.

## Product Goal

Interrupted or completed measurements remain useful. Users can recover context,
reopen data from Python, and export a measurement for offline analysis.

## MVP Scope

- Readable partial data and visible lifecycle state.
- Trash/recover instead of normal hard delete.
- Python reopen snippets using stable IDs.
- Measurement-centered export through SPEC-002.
- Privacy preview for sensitive provenance in exports.

## Not MVP

- Resumable managed execution.
- Full offline viewer polish before the write/reopen loop works.
- Importing old history as a built-in migration path.

## Key Stories

- US-006: Recover a partial measurement.
- US-008: Reopen measurement outputs from Python.
- US-009: Export a measurement for offline analysis.
