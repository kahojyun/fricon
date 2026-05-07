# SPEC-002: Measurement Export And Offline Analysis Design

## Status

Draft placeholder.

## Planning Boundary

Product and domain documents own scope while v0.2+ analysis is still in
progress. This design records a likely export direction only.

## Direction

Treat export bundles as read-only analysis packages:

```text
Measurement
  -> manifest and source identity
  -> produced dataset artifacts
  -> selected sample/session context
  -> notes/events and lifecycle state
  -> parameter/code/setup summaries selected for export
  -> checksums
  -> common tabular files where practical
  -> Python loader snippets
```

The Fricon manifest remains the source of meaning. CSV or similar common files
are convenience outputs and may be lossy.

## Open Questions

1. Bundle extension and physical layout.
2. Which common formats are required in v0.2.
3. How to represent variable-length traces and partial grids in common formats.
4. Direct Python reader API.
5. Privacy preview UX and defaults.
6. Whether Desktop opens bundles without a running local service in v0.2 or a
   later polish slice.
