# Current Dataset Semantics

## Status

Current implementation note. Update this when dataset semantic behavior,
writer metadata, reader interpretation, or chart projection semantics change.

## Purpose

This note describes the implemented dataset semantic model. Storage layout
details live in `dev-docs/current-storage-notes.md`. Historical rationale and
deferred design pressure live in
`dev-docs/dataset-semantic-architecture-proposal.md`.

## Implemented Model

New datasets created through ingest have a dataset-local semantic sidecar named
`dataset_manifest.json`. Semantic reads require this manifest and validate it
against the physical Arrow schema before exposing resolved interpretation.

The manifest v1 model includes:

- `manifest_version`
- `columns`
- optional `scan_plan`
- `realization`
- `inference`

Each manifest column records a Fricon dataset dtype and required semantic
facets:

- `value_kind`: numeric, categorical, boolean, timestamp, complex, or display
- `shape_kind`: scalar or trace
- `role`: value, logical_index, system, or display

Column metadata may also record `unit`, `label`, `hidden_by_default`, and
`chart_axis`. These are display and chart hints; renderer capabilities are
derived by interpretation rather than stored as durable manifest fields.

Fricon reserves the `__ds_` prefix for system fields. User payload columns and
scan-axis names using this prefix are rejected. New semantic datasets
physically materialize `__ds_record_id: uint64` as the first Arrow column. It
is monotonically increasing within the dataset, defines append order, and is
hidden from ordinary dataset reads, Python convenience reads, desktop details,
and chart defaults.

## Writer Behavior

Bare Python writes remain low-friction:

```python
with workspace.dataset_manager.create("run") as writer:
    writer.write(step=0, signal=1.0)
```

The first row freezes the user payload schema. Later rows must match that
schema. The writer constructs the storage schema by prepending
`__ds_record_id`, writes a minimal manifest, and appends Arrow chunk files.

Dataset creation also supports progressive semantic metadata:

- `columns=` for optional stored-column declarations and display/chart metadata
- `scan=` for optional logical scan axes
- `IndexAxis` or `None` for unknown-length integer scan axes
- `write_dict(..., logical_indices=...)` for explicit durable logical index
  positions

If `scan=` is present and no explicit logical indices are supplied on the first
write, the manifest usually uses implicit index realization. Mixed static and
unknown-length scan axes require sidecar realization, so callers must supply
`logical_indices` from the start. If `logical_indices` are supplied on the first
write, the dataset uses logical-index sidecar realization and writes
append-only sidecar chunks keyed by `__ds_record_id`.

The public `write(**kwargs)` path intentionally remains a clean payload-row
API. Advanced callers use `write_dict` when they need durable logical index
context.

## Reader And Interpretation Behavior

The semantic Rust reader requires `dataset_manifest.json`. It validates the
manifest against Arrow chunks, projects away hidden system columns for ordinary
reads, and exposes resolved interpretation through `DatasetReader::interpret`.

Resolved interpretation includes:

- physical and visible column ordinals
- semantic descriptors and renderer capability flags
- value columns
- inferred physical axes for minimal bare-write datasets
- manifest scan axes
- chart-axis candidate columns
- duplicate policy
- index realization

Minimal axis inference still exists for simple datasets, but only when the
manifest allows inference, no scan plan is present, and no explicit column
metadata is present. In that case the reader samples the first two visible rows
and marks leading numeric columns as inferred logical-index roles using the
temporary `row_order_placeholder` duplicate policy.

For datasets with scan plans, logical index points are resolved from either:

- implicit append order and scan shape, or
- `logical_index_chunk_<n>.arrow` sidecar chunks.

Chart projection appends resolved logical axes to projected batches and applies
the default duplicate policy `latest_by_record_id` for explicit scan semantics.

## UI And Chart Behavior

Desktop dataset detail uses resolved interpretation rather than raw Arrow field
order. It exposes visible columns with semantic descriptors, capability flags,
labels, units, hidden-by-default state, and chart-axis candidate state.

Chart semantics expose:

- duplicate policy
- index realization
- resolved axes
- value columns
- chart-axis candidates

Chart data loading uses semantic projection so logical scan axes and inferred
axes can participate in grouping, filtering, heatmap, and XY transforms without
being stored as ordinary user payload columns.

## Compatibility Notes

Manifest-free dataset payloads and earlier transition manifests without
required semantic facets are not supported by the semantic Rust read path.

Python `Dataset.to_arrow()` and `Dataset.to_polars()` are convenience helpers
that read Arrow chunk files directly and hide `__ds_` columns. They do not
perform full semantic manifest interpretation.

Dataset archives include catalog metadata, `dataset_manifest.json`, Arrow data
chunks, and logical-index sidecar chunks when present.

## Deferred

The current implementation does not make these concepts first-class:

- user-authored raw manifests
- manifest-owned saved views
- run or measurement manifests
- execution segment tables
- status-aware duplicate policies
- invalidation or quality-state semantics
- a public raw facts API that exposes system columns by default
- full semantic grid/select reader APIs beyond the current projection path
- null-heavy or late-appearing column workflows
