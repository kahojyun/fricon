# SPEC-002: Validation

## Status

Draft placeholder. Not implementation-ready.

## Planning Boundary

Validation scenarios are candidate checks for later export planning. Re-sync
them after product/domain scope and required ADRs are accepted.

## Scenario Checks

1. Export a measurement with one regular-grid dataset.
2. Export a partial-grid measurement and verify missing expected points remain
   visible.
3. Export an irregular/adaptive scan without forcing a fake dense grid.
4. Export variable-length traces with per-trace coordinates.
5. Open the export from Python without importing into a data library.
6. Verify sensitive provenance is omitted unless selected.
