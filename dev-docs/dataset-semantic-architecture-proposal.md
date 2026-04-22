# Dataset Semantics Architecture Proposal

## Status

Proposed.

This note revises the earlier append-only dataset idea for the actual Fricon
codebase and assumes the product is still pre-adoption, so breaking internal
changes are acceptable when they produce a cleaner long-term architecture.

## Goal

Replace the current inference-led dataset interpretation model with an explicit
dataset semantics layer while preserving the low-friction `write(col=...)`
workflow for simple cases.

The target architecture should:

- keep dataset payload storage append-only
- make dataset semantics explicit and local to the dataset
- stop treating chart semantics as a side effect of row adjacency heuristics
- keep dataset-local meaning separate from higher-level run or measurement
  context
- preserve the existing scalar, complex, and trace payload types as orthogonal
  to semantic interpretation

## Why Change

Today the dataset stack has one strong property and one weak property:

- Strong: payload storage is already effectively append-only and chunked on
  disk
- Weak: plotting and live-monitor semantics are inferred indirectly from the
  first scalar columns and recent row ordering

That weak property leaks through the stack:

- the backend derives "index columns" from the first two rows
- dataset detail returns only a coarse `isIndex` bit
- frontend chart selection and live grouping use that bit as the primary
  semantic signal
- non-contiguous scans, retries, resumes, and explicit scan plans are not
  first-class concepts

This is the wrong abstraction boundary. Storage should own facts. A semantics
layer should own interpretation. Chart code should consume resolved semantics,
not rediscover them.

## Architectural Decision

Treat each dataset as three layers:

```text
dataset facts      -> append-only Arrow chunk files
dataset semantics  -> explicit manifest sidecar
dataset projections -> resolved chart/grid/live-view interpretation
```

The important shift is that projections no longer infer meaning directly from
raw rows when explicit semantics are available.

## Proposed Dataset Layout

Use the real Fricon chunked storage model as the baseline rather than an
invented single-file payload:

```text
workspace/
  data/
    <uid[0:2]>/
      <uid>/
        dataset_manifest.json
        data_chunk_0.arrow
        data_chunk_1.arrow
        ...
```

Notes:

- File names can still be revised, but the proposal assumes chunked Arrow IPC
  files remain the physical storage substrate.
- SQLite dataset metadata remains separate and continues to own workspace-level
  catalog concerns such as name, description, tags, favorite state, and status.
- A higher-level run or measurement manifest is explicitly out of scope for
  this proposal.

## Core Invariants

### 1. Rows Are Append-Only Facts

Each stored row records an observation that happened. Rows are not overwritten
in place to represent retries, resumes, or corrections.

### 2. Schema Is Fixed At Dataset Creation

Fricon currently fixes schema from the first row and requires later rows to
match exactly. The clean architecture should keep the "fixed schema" invariant,
but move schema choice to dataset creation rather than accidental first-row
shape.

Implication:

- any system columns that may appear later must be declared up front
- optional system columns should be nullable, not absent from some rows

This avoids hidden schema drift and keeps the write path simple.

### 3. `__ds_` Is Reserved For System Fields

Use a reserved internal prefix for Fricon-owned columns:

```text
__ds_
```

Examples:

- `__ds_point_id`
- `__ds_time_ns`
- `__ds_idx_x`
- `__ds_idx_y`
- `__ds_sweep_id`
- `__ds_frame_id`
- `__ds_status`

Collision with user columns should be a hard creation-time error.

### 4. `__ds_point_id` Is Mandatory For New Semantic Datasets

New-format datasets should always materialize:

```text
__ds_point_id: uint64
```

Semantics:

- monotonically increasing within the dataset
- stable across restarts, export, and import
- defines append order
- is the tie-breaker for duplicate logical positions

### 5. Manifest Is Canonical For Dataset Meaning

The manifest, not Arrow field order and not chart heuristics, is the canonical
source for:

- column roles
- axis identity
- logical index mapping
- scan-plan hints
- live grouping semantics
- default view suggestions

### 6. Compatibility Inference Is A Fallback Layer

Inference still exists, but only as a compatibility mechanism for:

- bare datasets created without semantic configuration
- old test fixtures
- exploratory one-off data

Inference should not remain the primary path for new semantic datasets.

The proposal does not require that inferred semantics remain in-memory only. A
future implementation may materialize a generated manifest for caching or user
enrichment, but downstream consumers should still depend only on the resolved
semantic model.

## Semantic Model

The manifest should stay small and focused on dataset-local meaning.

Recommended top-level sections:

- `manifest_version`
- `columns`
- `scan_plan`
- `realization`
- `live_grouping`
- `view_defaults`
- `compatibility`

### Semantic Roles Are Orthogonal To Data Types

The semantic layer describes what a column means in the experiment, not how its
payload is encoded inside a row.

This proposal should preserve Fricon's existing data-type distinctions:

- scalar columns
- complex scalar columns
- trace columns

Examples:

- a `measurement` may be a scalar or a trace
- a `scan_axis` is usually scalar
- grid and live-grouping semantics describe row-to-row structure, not the
  internal X axis of a trace payload

This separation avoids forcing scan semantics and trace rendering into the same
abstraction.

### Column Semantics

Each column may declare:

- `role`
- `label`
- `unit`
- `hidden_by_default`
- `axis_id`

Recommended v1 role vocabulary:

- `point_id`
- `scan_axis`
- `logical_index`
- `measurement`
- `timestamp`
- `sweep_id`
- `frame_id`
- `status`
- `categorical`
- `annotation`
- `unknown`

Do not add a large ontology in v1.

### Scan Plan

`scan_plan` should be lightweight and permissive. It is a hint layer, not a
full planner state dump.

Recommended fields:

- `axes`
- `traversal`
- `planned_shape`
- `strictness`
- `allows_partial`
- `allows_duplicate_positions`

Each axis should describe:

- axis identity
- value column
- optional logical index column
- mode: `linear`, `list`, `categorical`, `adaptive`, `implicit_index`,
  `unknown`

### Realization

`realization` should describe dataset facts that affect interpretation, not
mutable summaries that are easy to let drift.

Recommended v1 fields:

- `append_only: true`
- `point_id_column`
- `duplicate_resolution_default`

Avoid storing counters such as `actual_points` in the manifest unless there is
clear write ownership and consistency logic.

### Live Grouping

Live grouping must become explicit.

Recommended fields:

- `sweep_id_column`
- `frame_id_column`
- `fallback_group_by_axes`

Critical rule:

- non-contiguous scans should require explicit `sweep_id` or `frame_id` for
  correct live grouping
- adjacency-based grouping remains valid only for contiguous legacy scans

### View Defaults Versus Saved Views

The manifest may contain dataset-owned default or recommended views, for
example:

- default heatmap quantity
- default scan axes
- default duplicate resolution

It should not be the primary home for user-owned saved UI views such as:

- manually saved chart presets
- personal filter sets
- temporary viewport or zoom state

Those belong more naturally to workspace or application state. This proposal is
about portable dataset semantics, not all user interface preferences.

## Deliberate Separation Of Concerns

### Dataset Storage Owns Facts

`crates/fricon/src/dataset/storage/*` should own:

- chunk naming and layout
- Arrow read and write
- append persistence
- payload integrity

It should not own semantic interpretation rules.

### Dataset Semantics Owns Meaning

Introduce a new semantic slice under `crates/fricon/src/dataset/semantics/`
that owns:

- manifest types
- manifest read and write
- validation
- defaulting
- compatibility inference for bare datasets

This layer should not know about charts or Tauri.

### Dataset Interpretation Owns Resolved Projections

Introduce a resolved interpretation layer under
`crates/fricon/src/dataset/interpret/` that turns raw schema plus manifest into
stable, chart-ready semantics.

It should expose a resolved model such as:

- scan axes
- logical index columns
- measurement candidates
- live grouping rules
- default views
- duplicate resolution policy

This becomes the API consumed by UI query code and chart transforms.

### Charts Consume Resolved Semantics

Chart selection should stop depending on the coarse `isIndex` bit as its
primary abstraction.

Instead, `crates/fricon-ui` should receive richer dataset interpretation data,
for example:

- `role`
- `isMeasurement`
- `isScanAxis`
- `isLogicalIndex`
- `axisId`
- `hiddenByDefault`
- resolved live grouping defaults
- resolved default views

The current `isIndex` field may exist temporarily as a compatibility projection,
but it should not remain the canonical contract.

## Writer Architecture

### Creation Phase

Dataset creation should become explicit even for the simple API.

Creation computes and freezes:

- payload schema
- selected system columns
- initial manifest

Suggested API shape:

```python
with manager.create(
    "name",
    semantics=DatasetSemantics(
        columns=...,
        scan_plan=...,
        system_fields=...,
    ),
) as writer:
    ...
```

For bare writes:

- Fricon still allows `write(col=...)`
- the first row may still seed user columns for convenience
- but the writer, not the chart layer, is responsible for constructing a
  minimal semantic manifest

### Write Phase

Each row writes:

- user fields
- nullable system fields declared at creation

Suggested ergonomic Python surface:

```python
writer.write(
    x=0.1,
    y=2.0,
    signal=0.95,
    ds={
        "idx": {"x": 10, "y": 2},
        "sweep_id": 7,
    },
)
```

The `ds` helper is only API sugar. Internally it maps to fixed nullable
`__ds_*` columns.

## Reader Architecture

The reader should expose both facts and resolved meaning.

Suggested responsibilities:

- `reader.points()` -> raw row access
- `reader.manifest()` -> raw manifest
- `reader.interpret()` -> resolved dataset semantics
- `reader.grid(value=...)` -> grid projection using resolved interpretation
- `reader.live_groups(k=...)` -> live grouping using resolved semantics

Important rule:

- duplicate grid placement must be resolved in the interpretation layer, not
  separately reimplemented in each chart path

## Duplicate And Status Semantics

Duplicate logical positions are allowed and expected.

Recommended v1 rule:

- default duplicate policy: `latest_by_point_id`

Do not use `latest_valid_by_point_id` in v1 unless status semantics are made
mandatory and precisely defined.

If status support is needed, define a compact status vocabulary first:

- `valid`
- `invalid`
- `aborted`
- `unknown`

## Live Monitor Rules

### Contiguous Legacy Scans

Allowed fallback:

- derive sweeps from slow-axis tuple changes
- derive live heatmap frame from the last contiguous outer group

### Semantic Datasets

Preferred path:

- use explicit `sweep_id`
- use explicit `frame_id`
- use manifest-configured fallback grouping only when scans are still
  contiguous

### Non-Contiguous Scans

For shuffled scans, resumes, ROI-first acquisition, or retries:

- explicit grouping IDs are required for correctness
- adjacency heuristics are not sufficient

This should be documented as a hard semantic contract, not a best-effort hope.

## V1 Scope

The clean v1 should include:

- chunked append-only Arrow payloads
- `dataset_manifest.json`
- `__ds_point_id`
- explicit column roles
- optional logical index columns
- scan-plan-lite
- explicit live grouping fields
- resolved interpretation API
- chart integration that prefers resolved semantics over inference

The clean v1 should defer:

- measurement or run manifest design
- execution segment tables
- rich provenance graph
- expanded planned-point tables
- multiple duplicate-resolution policies
- user-editable manifest history

## Worked Use Cases

This section pressure-tests the proposal against the two most important API
shapes:

- direct user scripting for simple experiments
- fully wired execution through a higher-level experiment system

The goal is to keep both paths clean without forcing the same amount of
ceremony onto both.

### Use Case 1: Direct Pythonic Scanning

Scenario:

- the user writes a small script directly against the public Python API
- the scan is a simple nested loop
- there is no jump scan, retry, resume, or adaptive planner
- the user wants to start quickly and does not want semantic boilerplate up
  front

Example user code:

```python
from fricon import Workspace

ws = Workspace.connect(".dev/ws")

with ws.dataset_manager.create("quick_iv_map") as ds:
    for ix, gate_v in enumerate([-0.2, -0.1, 0.0, 0.1, 0.2]):
        set_gate(gate_v)
        for bias_idx, bias_v in enumerate([0.0, 0.01, 0.02]):
            set_bias(bias_v)
            current = measure_current()
            ds.write(
                gate_v=gate_v,
                bias_v=bias_v,
                current_a=current,
            )
```

Desired behavior:

- this remains valid and low-friction
- the user is not required to declare scan axes, logical indices, or view
  definitions before the first row
- Fricon still creates a semantic dataset, but it does so with conservative
  defaults

Recommended internal behavior:

1. The first row determines the user column set and data types.
2. Fricon materializes `__ds_point_id` automatically.
3. Fricon writes a minimal manifest with:
    - all user columns marked as `unknown`
    - `__ds_point_id` marked as `point_id`
    - compatibility mode enabled
4. Readers and charts may still use compatibility inference for this dataset.

Result:

- the user gets the same "just start writing rows" experience as today
- the dataset is still owned by the new architecture
- no user-visible semantic burden is introduced for the simple path

Optional upgrade path:

If the user wants slightly better semantics without much extra ceremony, the
API should allow a small amount of opt-in metadata:

```python
with ws.dataset_manager.create(
    "quick_iv_map",
    semantics={
        "columns": {
            "gate_v": {"role": "scan_axis", "axis_id": "gate", "unit": "V"},
            "bias_v": {"role": "scan_axis", "axis_id": "bias", "unit": "V"},
            "current_a": {"role": "measurement", "unit": "A"},
        }
    },
) as ds:
    for ix, gate_v in enumerate([-0.2, -0.1, 0.0, 0.1, 0.2]):
        for bias_idx, bias_v in enumerate([0.0, 0.01, 0.02]):
            ds.write(gate_v=gate_v, bias_v=bias_v, current_a=measure_current())
```

This is still small enough for a Python user to tolerate. The key is that the
semantic path is available but not mandatory.

### Use Case 2: Fully Wired Experiment System

Scenario:

- the user works inside a higher-level experiment system
- scan configuration, loop structure, planner state, and system columns are
  wired by the experiment runtime
- the experiment author should focus on semantic intent and the loop body, not
  on dataset plumbing

Recommended split of responsibility:

- the experiment system owns:
    - scan specification
    - traversal policy
    - retry or resume policy
    - grouping IDs
    - loop orchestration
- the experiment author owns:
    - semantic declarations that matter to the experiment
    - the loop body function or measurement function

Example shape:

```python
@experiment(
    dataset={
        "name": "transport_map",
        "columns": {
            "gate_v": {"role": "scan_axis", "axis_id": "gate", "unit": "V"},
            "field_t": {"role": "scan_axis", "axis_id": "field", "unit": "T"},
            "signal_v": {"role": "measurement", "unit": "V"},
        },
    },
    scan={
        "axes": [
            {"id": "gate", "values": gate_points},
            {"id": "field", "values": field_points},
        ],
        "traversal": {"fast_axis": "gate", "slow_axes": ["field"]},
    },
)
def run_point(ctx):
    set_gate(ctx.axis("gate").value)
    set_field(ctx.axis("field").value)
    return {"signal_v": lockin_read()}
```

Recommended runtime behavior:

1. The experiment system computes the full dataset schema before acquisition
   starts.
2. It requests the system fields it needs up front, for example:
    - `__ds_point_id`
    - `__ds_idx_gate`
    - `__ds_idx_field`
    - `__ds_sweep_id`
    - `__ds_frame_id` when relevant
    - `__ds_status` if retries or invalidation are supported
3. It creates the manifest before the first data row is written.
4. Each loop iteration writes one row with both user values and system values.

The actual write path may look like:

```python
ds.write(
    gate_v=ctx.axis("gate").value,
    field_t=ctx.axis("field").value,
    signal_v=lockin_read(),
    ds={
        "idx": {
            "gate": ctx.axis("gate").index,
            "field": ctx.axis("field").index,
        },
        "sweep_id": ctx.sweep_id,
        "frame_id": ctx.frame_id,
        "status": "valid",
    },
)
```

This is acceptable because it is not end-user boilerplate. It is runtime-owned
plumbing inside the full experiment stack.

Result:

- the experiment system gets exact semantic control
- live monitor and chart behavior become correct for non-trivial scans
- retries, resumes, and non-contiguous acquisition can be represented without
  hacking chart heuristics
- the experiment author only writes semantic intent plus measurement logic

## API Design Implication

The public API should intentionally support two entry modes:

- minimal mode:
    - optimized for ad hoc Python scripts
    - zero or near-zero semantic boilerplate
    - compatibility inference remains acceptable
- semantic mode:
    - optimized for framework-owned execution
    - semantics declared before acquisition starts
    - fixed schema, fixed system columns, explicit grouping, explicit scan plan

These modes should converge on the same storage and interpretation model. They
should differ only in how much information is supplied at creation time.

## Breaking Changes That Are Worth Taking Now

Because the product is still pre-adoption, these changes are acceptable:

- change dataset payload layout inside the workspace data directory
- add a workspace compatibility bump if needed
- replace the dataset detail UI contract with richer semantic fields
- remove the assumption that chart semantics come from inferred "index
  columns"

These are precisely the kinds of changes that become expensive after adoption
and are cheap now.

## Migration Strategy

This proposal does not optimize for long-lived backward compatibility.

Recommended approach:

1. Introduce the semantic model and resolved interpretation API.
2. Switch chart and live-query code to consume resolved interpretation.
3. Update dataset creation to produce manifests for all new datasets.
4. Keep a compatibility inference path only for old fixtures and ad hoc test
   data.
5. Remove architecture that treats inferred `isIndex` as the primary semantic
   contract once the new path is stable.

## Suggested Module Plan

Rust backend:

```text
crates/fricon/src/dataset/
  storage/        # Arrow chunk IO and filesystem layout
  semantics/      # manifest model, load/save, validation
  interpret/      # resolved dataset meaning for consumers
  ingest/         # write workflow and schema/materialization rules
  read/           # row access plus interpretation entry points
```

Desktop UI backend:

```text
crates/fricon-ui/src/features/datasets/
  queries.rs      # return richer semantic detail, not only isIndex
```

Charts:

```text
crates/fricon-ui/src/features/charts/
  transform/      # consume resolved semantics instead of rediscovering them
```

Frontend:

```text
frontend/src/features/datasets/api/
  types.ts        # richer semantic DTOs

frontend/src/features/charts/model/
  chartViewerLogic.ts  # choose defaults from resolved semantics
```

## Acceptance Criteria

The architecture is considered successful when:

- a known 2D scan opens correctly from explicit semantics without relying on
  first-two-row inference
- shuffled scans with logical indices render to the correct grid cells
- retries and resumes preserve all fact rows while default projections use the
  latest row by `__ds_point_id`
- live views for non-contiguous scans behave correctly when explicit grouping
  IDs are present
- bare `write(col=...)` still works, but now produces a minimal semantic
  manifest owned by the dataset layer

## Summary

The clean direction for Fricon is not "better index inference". It is:

- append-only facts in Arrow chunks
- explicit dataset semantics in a manifest
- a resolved interpretation layer between storage and charts

That gives Fricon the right long-term boundary:

- storage code owns persistence
- semantics code owns meaning
- chart code owns rendering

This is a larger change than an additive sidecar, but it is the correct change
to make before the product hardens around the current inference-led model.
