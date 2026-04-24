# Dataset Semantics Implementation Plan

## Status

Proposed.

This plan implements the direction in
`dev-docs/dataset-semantic-architecture-proposal.md`. The goal is to land the
minimal durable architecture first, then incrementally add richer scan,
sidecar, API, and UI behavior.

## Implementation Principles

- Start with the smallest durable manifest that every new dataset can own.
- Keep payload storage append-only and continue using chunked Arrow IPC files.
- Keep Arrow schema in the create stream payload; use create metadata only for
  catalog metadata and optional semantic creation metadata.
- Treat manifest JSON as a Rust serde-maintained format, not a user-authored
  config file.
- Keep validation separate from serde deserialization.
- Keep compatibility inference available while consumers migrate off
  `DatasetReader::index_columns()`.

## Phase 1: Manifest Foundation

Add a new `crates/fricon/src/dataset/semantics/` slice that owns manifest
types, file IO, validation, and minimal-manifest construction.

The initial manifest implementation should include:

- `DatasetManifest` with required `manifest_version`, `columns`,
  `realization`, and `compatibility` fields.
- Optional `scan_plan`, `live_defaults`, and `view_defaults` fields with serde
  defaults and `skip_serializing_if`.
- Serde-friendly tagged enums for durable values:
    - dataset datatype, for example `{ "kind": "float64" }`
    - system column kind, for example `{ "kind": "point_id" }`
    - index realization, starting with `{ "kind": "none" }`
    - duplicate policy, starting with `{ "kind": "latest_by_point_id" }`
- A `dataset_manifest.json` layout helper colocated with storage layout naming.
- Atomic manifest writes through a temporary file in the dataset directory.

Validation must check:

- `manifest_version` is supported.
- `columns` contains the configured `point_id_column`.
- `__ds_point_id` is a `uint64` system `point_id` column in minimal manifests.
- user-visible columns do not use the reserved `__ds_` prefix.
- `duplicate_resolution_default` is `latest_by_point_id` in v1.
- `index_realization.kind = "none"` does not claim sidecar state.

Do not use `deny_unknown_fields` in the durable manifest structs. Rejecting
future manifests should be an explicit version/validation decision.

## Phase 2: Ingest And Storage Integration

Make every new dataset write a manifest and materialize `__ds_point_id`.

Bare write behavior:

1. Python or Rust client creates a dataset as it does today.
2. First payload row still freezes the user payload schema in minimal mode.
3. Ingest augments the stored Arrow schema and every row with `__ds_point_id`.
4. Ingest writes `dataset_manifest.json` before finalizing the first chunk.
5. Finalized datasets always have both payload chunks and a manifest.

Implementation details:

- Reject user columns whose names start with `__ds_` before storage writes
  begin.
- Assign `__ds_point_id` monotonically from zero in append order.
- Include `__ds_point_id` in read schemas for semantic datasets.
- Preserve existing abort behavior: rows successfully written before abort stay
  on disk and the manifest remains valid for those rows.
- Keep sidecar logical-index files out of this phase.

Portability must be updated in this phase or the same change series:

- export archives include `dataset_manifest.json`.
- import staging preserves `dataset_manifest.json`.
- import preview remains based on SQLite metadata, not manifest details.

Decide whether `WORKSPACE_VERSION` must bump before merge. If mandatory
manifests make old workspaces incompatible with new read paths, bump the
workspace version and document the migration behavior. If all old datasets still
read through compatibility inference, do not bump solely for writing new
manifests.

## Phase 3: Resolved Interpretation

Add `crates/fricon/src/dataset/interpret/` to resolve dataset facts into a
stable consumer model.

The resolved model should expose:

- data columns with dtype, label, unit, hidden state, and chart-axis candidate
  metadata.
- system columns such as `__ds_point_id`.
- logical index realization: `none`, `implicit`, or later `sidecar`.
- duplicate resolution policy.
- compatibility-derived index columns when no durable scan/index metadata
  exists.

Reader behavior:

- Add reader entry points for raw manifest and resolved interpretation.
- Keep `DatasetReader::index_columns()` temporarily as a compatibility adapter
  backed by resolved interpretation where possible.
- For old datasets with no manifest, synthesize an in-memory minimal
  interpretation from schema plus legacy inference.
- Chart/grid code must use interpretation-layer duplicate handling rather than
  reimplementing duplicate selection.

Acceptance for this phase:

- existing datasets without manifests still open.
- new minimal-manifest datasets open without first-two-row inference being the
  canonical path.
- existing chart behavior remains equivalent for current fixture datasets.

## Phase 4: Scan Plans And Logical Index Sidecars

Add explicit scan support after minimal manifests and resolved interpretation
are stable.

Scan plan support:

- Add optional `scan_plan` manifest structs for axes, traversal, partial scans,
  and index realization.
- Support unknown-length integer axes for minimizer-style workflows.
- Support static axis coordinate lists in the manifest.
- Do not require one-to-one mapping between scan axes and payload columns.
- Do not add a separate durable derived-coordinate model; materialized payload
  columns can be marked as chart-axis candidates.

Sidecar support:

- Add append-only `index_chunk_*.arrow` files only when explicit logical
  indices are provided.
- Sidecar schema is `__ds_point_id: uint64` plus one `int64` column per logical
  axis.
- Sidecar chunks follow payload chunk naming and atomic write discipline.
- `index_realization.kind = "sidecar"` records the sidecar naming pattern and
  logical axis order.

Reader/query behavior:

- `reader.indexes()` resolves implicit indices from append order and scan
  traversal, or reads sidecar indices when present.
- `reader.select(index={...})` filters by logical index coordinates.
- ragged grids are represented as observed sparse index pairs, not forced into
  rectangular planned shapes.

## Phase 5: Public API And Transport

Expose progressive Python and Rust client APIs once the backend can persist and
read the corresponding semantics.

Python API:

- Extend `DatasetManager.create()` with optional keyword-only `columns=None`
  and `scan=None`.
- Keep `write(**kwargs)` unchanged.
- Extend `write_dict(values, *, logical_indices=None)` for framework-owned
  explicit logical indices.
- Add typed helper stubs for `Column`, `IndexAxis`, `ColumnSpec`, and
  `ScanAxisSpec` in `crates/fricon-py/python/fricon/_core.pyi`.
- Runtime may accept plain mappings, but documented examples should prefer
  typed helpers where they improve completion.

Transport:

- Keep Arrow schema in stream payload bytes.
- Extend `CreateMetadata` only with optional semantic creation metadata needed
  by `columns=` and `scan=`.
- Bump `IPC_PROTOCOL_VERSION` in the same change that changes protobuf
  request/response contracts.
- Regenerate affected protobuf bindings and TypeScript bindings as part of that
  change.

## Phase 6: UI, Charts, And Live Views

Move UI and chart code from inferred `isIndex` toward resolved interpretation.

Dataset detail contract:

- Add interpretation fields such as `isLogicalIndex`, `isChartAxisCandidate`,
  `label`, `unit`, and resolved scan axes.
- Keep `isIndex` temporarily as a compatibility projection.
- Update generated frontend bindings after the Tauri DTO changes.

Chart behavior:

- Chart selection defaults should prefer resolved scan/index metadata.
- Heatmap transforms should consume resolved logical indices and duplicate
  policy.
- Ragged heatmaps should render from observed sparse index pairs.
- Live views may use active write-session grouping for refresh timing without
  persisting `sweep_id` or `frame_id`.

Once chart paths no longer depend on `isIndex`, remove direct use of
`DatasetReader::index_columns()` from UI/chart code.

## Testing Plan

Rust:

- manifest serde round trips for minimal manifests and optional sections.
- manifest validation failures for missing point id, reserved user prefixes,
  unsupported version, and invalid realization combinations.
- ingest tests proving bare writes create `dataset_manifest.json`, append
  `__ds_point_id`, and reject user `__ds_` columns.
- reader tests for manifest loading, compatibility fallback, resolved
  interpretation, duplicate policy, implicit indices, sidecar indices, and
  ragged grids.
- portability tests proving manifests and sidecar chunks round trip through
  export/import.

Python:

- existing `write(**kwargs)` behavior still works.
- optional `columns=` can declare dtype/unit/label/chart-axis hints.
- optional `scan=` can declare static axes and `IndexAxis`.
- `write_dict(..., logical_indices=...)` persists sidecar-backed indices when
  sidecar support lands.

Frontend/UI:

- API normalization tests for new dataset detail fields.
- chart model tests proving defaults come from resolved interpretation rather
  than `isIndex`.
- browser tests for opening minimal-manifest datasets and explicit 2D scan
  datasets.

Quality gates:

- Rust-focused phases: `cargo nextest run --workspace` or a targeted nextest
  subset while iterating.
- Python API phases: `uv run maturin develop` before `uv run pytest` when
  bindings change.
- Frontend phases: `pnpm run check`, plus targeted `pnpm run test:unit` or
  `pnpm run test:browser` for chart/UI changes.

## Implementation Order And Cut Points

Preferred PR sequence:

1. Manifest serde model, IO helpers, validation, and tests.
2. Ingest integration for minimal manifests and `__ds_point_id`.
3. Reader interpretation layer and compatibility fallback.
4. Portability updates for manifests.
5. Optional `columns=` creation metadata.
6. Optional `scan=` and implicit index realization.
7. Sidecar logical index storage and `write_dict(..., logical_indices=...)`.
8. UI/detail DTO expansion and frontend binding updates.
9. Chart transform migration to resolved interpretation.
10. Cleanup of legacy `isIndex` as the canonical chart contract.

Each cut point should preserve current simple dataset creation and reading.
Avoid landing UI-breaking DTO changes before generated bindings and frontend
normalization are updated in the same PR.
