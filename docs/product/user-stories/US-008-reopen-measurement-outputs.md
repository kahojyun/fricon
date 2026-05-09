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

- Desktop and CLI can copy stable measurement and dataset IDs as a fast path.
- Desktop and CLI can show copyable Python reopen snippets.
- Public read APIs can open measurements, dataset artifacts, and partial data
  by stable IDs.
- Reopen snippets are allowed to be advanced actions if copying the stable ID
  into an existing reader call is the normal analyst workflow.
- Reopened data preserves scan schema, units, labels, lifecycle state, and
  partial-data semantics.
- Reopened data can be converted into analysis-friendly Python objects such as
  NumPy, pandas, or Polars where appropriate for the dataset shape.
- Trace-bearing datasets can expose multiple analysis views where appropriate:
  record-centric nested trace tables, exploded trace tables, sample-level long
  tables, ndarray-like arrays, and xarray-like labeled views when the data is
  rectangular enough.
- IQ single-shot datasets can be read as ndarray-like data with sweep
  dimensions before the shot dimension when shape permits.
- Minimizer or optimizer step records can be read as a step table, with a
  convenient best-parameter summary by outer sweep condition when meaningful.
- Reader views preserve enough trace, axis, segment, and derived-channel
  metadata to support line plots, coarse/fine trace comparison or
  concatenation, 2D heatmaps, and complex magnitude/phase views without
  re-parsing storage paths.
- Path-based access is not the documented normal path.

## Not In Scope

- Treating the private storage layout as a public API.
- Importing old legacy-system data before new Fricon outputs can be reopened.
- Committing the internal storage model to a specific analysis framework.
- Replacing downstream notebooks, fitting code, or classifier-tuning scripts in
  the first adoption slice.

## Related Capabilities

CAP-010, CAP-026.
