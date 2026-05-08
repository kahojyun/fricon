# EPIC-004: Recovery, Reopen, And Export

## Status

Draft pending product-analysis revalidation.

## Product Goal

Interrupted or completed measurements remain useful. Users can recover context,
reopen data from Python, and export a measurement for offline analysis.

## Initial Adoption Scope

- Readable partial data and visible lifecycle state.
- Trash/recover instead of normal hard delete.
- Python reopen snippets using stable IDs.
- Measurement-centered read-only export bundles.
- Lightweight Python reader for exported bundles without importing into another
  editable library or running the acquisition-time local runtime.
- Analysis-friendly reads into common Python objects where appropriate.
- Human-readable export manifest or index preview.
- Privacy preview for sensitive provenance in exports.

## Not Initial Adoption

- Resumable managed execution.
- Full offline viewer polish before the write/reopen loop works.
- Importing old history as a built-in migration path.
- First-slice report or presentation generation.
- Mandatory generic CSV, Parquet, or NumPy exports before demand is validated.

## Key Stories

- US-006: Recover a partial measurement.
- US-008: Reopen measurement outputs from Python.
- US-009: Export a measurement for offline analysis.
