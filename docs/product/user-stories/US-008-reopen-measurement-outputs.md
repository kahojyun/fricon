# US-008: Reopen Measurement Outputs From Python

## Status

Draft pending product-analysis revalidation.

## Primary Epic

EPIC-004: Recovery, annotation, reopen, and export.

## Story

As an Experimental Data Analyst, I want to reopen a measurement or one of its
dataset artifacts from Python by stable ID so that later analysis does not
depend on remembering storage paths.

## Success Criteria

- Desktop and CLI can show copyable Python reopen snippets.
- Public read APIs can open measurements, dataset artifacts, and partial data
  by stable IDs.
- Reopened data preserves scan schema, units, labels, lifecycle state, and
  partial-data semantics.
- Reopened data can be converted into analysis-friendly Python objects such as
  NumPy, pandas, or Polars where appropriate for the dataset shape.
- Path-based access is not the documented normal path.

## Not In Scope

- Treating the private storage layout as a public API.
- Importing old legacy-system data before new Fricon outputs can be reopened.

## Related Capabilities

CAP-010, CAP-026.
