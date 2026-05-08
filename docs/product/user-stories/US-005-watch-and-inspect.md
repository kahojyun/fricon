# US-005: Watch And Inspect Data

## Status

Draft pending product-analysis revalidation.

## Primary Epic

EPIC-003: Measurement console and inspection.

## Story

As a Measurement Run Operator, I want local live and historical table/chart
views so that I can decide whether a measurement is working.

## Success Criteria

- Desktop starts from a measurement console.
- Live views are noncritical consumers and cannot block writes.
- Core views are table, line/scatter, basic heatmap, and simple trace
  inspection.
- Users can keep more than one relevant measurement, table, plot, or trace view
  visible while acquisition continues.
- Users can open produced datasets directly when needed.
- Failed, interrupted, or partial runs are visibly different from completed
  runs.
- Stale, lagging, or disconnected live views are visible as inspection state,
  not as write failures.

## Not In Scope

- Publication plotting, generic dashboard building, or rich comparison views.

## Related Capabilities

CAP-007, CAP-008, CAP-026.
