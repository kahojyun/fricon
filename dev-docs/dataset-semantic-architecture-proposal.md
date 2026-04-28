# Dataset Semantics Architecture Proposal

## Status

Proposed.

The durable foundation decisions for `dataset_manifest.json`,
`__ds_record_id`, the reserved `__ds_` prefix, v1 dtypes, plain Arrow storage,
compatibility fallback, and the interpretation boundary are accepted in
`dev-docs/adr/0002-decide-dataset-semantic-manifest-v1.md`.

This is not current behavior. Do not implement or document behavior from this
proposal without checking current code, `dev-docs/current-storage-notes.md`, and
the relevant checklist in `dev-docs/maintenance-checklist.md`.

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
- keep scalar, complex, and trace payload types as the primary description of
  column kind, with metadata used only for display and chart hints

## Pre-Adoption Breaking Change Policy

Fricon has not reached production adoption yet. Use that timing to clean up
internal storage and semantic contracts now instead of preserving legacy
inference paths as long-term architecture.

Feature 1 may make breaking changes to workspace format, dataset payload
layout, manifest requirements, protocol contracts, generated bindings, and
desktop detail DTOs when those changes simplify the durable semantics model.
When a compatibility bump is needed, make it explicit through the normal
workspace, IPC, release, and maintenance checklists.

Breaking changes are appropriate for:

- making `dataset_manifest.json` mandatory for new datasets
- materializing `__ds_record_id` in new datasets
- reserving `__ds_` for Fricon-owned system fields
- replacing Fricon Arrow extension metadata with plain Arrow physical schemas
  plus manifest-owned semantics for new datasets
- making resolved interpretation, not `isIndex`, the canonical consumer
  contract
- changing internal chunk layout, DTOs, or protocol versions when they would
  otherwise preserve the wrong abstraction boundary

Do not use pre-adoption cleanup as a reason to break the simple Python user
model. Bare `write(col=...)` and first-row schema inference in minimal mode
should keep working. Users should not need to author raw manifests, declare
scan plans, or understand storage details for simple datasets.

Compatibility fallback should exist to keep old fixtures and local test data
readable during the transition. It should not constrain the new semantic model
or remain the primary path for new datasets.

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
        index_chunk_0.arrow      # optional logical-index sidecar
        index_chunk_1.arrow      # optional logical-index sidecar
        ...
```

Notes:

- File names can still be revised, but the proposal assumes chunked Arrow IPC
  files remain the physical storage substrate.
- Logical index sidecar chunks are optional. Regular ordered scans should not
  pay a main-payload storage cost for indices that can be derived from append
  order and scan shape.
- SQLite dataset metadata remains separate and continues to own workspace-level
  catalog concerns such as name, description, tags, favorite state, and status.
- A higher-level run or measurement manifest is explicitly out of scope for
  this proposal.

## Core Invariants

### 1. Rows Are Append-Only Facts

Each stored row records an observation that happened. Rows are not overwritten
in place to represent retries, resumes, or corrections.

### 2. Schema Is Fixed Once Writing Starts

Fricon currently fixes schema from the first row and requires later rows to
match exactly. The clean architecture should keep the "fixed schema" invariant,
while allowing the simple API to infer schema from the first written row.
Semantic mode may declare columns earlier; minimal mode freezes payload columns
once writing starts.

Implication:

- payload columns are fixed once the dataset starts writing
- `__ds_record_id` is always materialized by the writer
- durable logical indices live in an index sidecar when they cannot be derived
  from append order
- optional future system payload columns should be declared explicitly rather
  than appearing mid-stream

This avoids hidden schema drift and keeps the write path simple.

For v1, null-heavy workflows are deliberately out of scope:

- the first written row should contain all payload columns
- values should be non-null
- nullable or late-appearing columns should require an explicit future API

### 3. `__ds_` Is Reserved For System Fields

Use a reserved internal prefix for Fricon-owned columns:

```text
__ds_
```

Examples:

- `__ds_record_id`
- `__ds_recorded_at`

Collision with user columns should be a hard creation-time error.

### 4. `__ds_record_id` Is Mandatory For New Semantic Datasets

New-format datasets should always materialize:

```text
__ds_record_id: uint64
```

Semantics:

- monotonically increasing within the dataset
- stable across restarts, export, and import
- defines append order
- is the tie-breaker for duplicate logical positions

`__ds_recorded_at` is optional in v1. When present, it should use Fricon's
timestamp business dtype with microsecond precision, mapped internally to the
appropriate Arrow timestamp representation.

### 5. Manifest Is Canonical For Dataset Meaning

The manifest, not Arrow field order and not chart heuristics, is the canonical
source for:

- column metadata and display hints
- axis identity
- logical index mapping
- scan-plan hints
- active live grouping defaults
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
- `realization`
- `compatibility`
- `scan_plan`
- `live_defaults`
- `view_defaults`

Only the first four sections are required for the minimal manifest. Other
sections should be added only when the user or a higher-level system supplies
the relevant information, or when an implementation needs to cache a resolved
interpretation.

### Minimal Manifest

Bare `write(col=...)` should create a small manifest that records only the
durable facts needed by all datasets:

```json
{
    "manifest_version": 1,
    "columns": {
        "__ds_record_id": {
            "dtype": {
                "kind": "uint64"
            },
            "system": {
                "kind": "record_id"
            }
        },
        "x": {
            "dtype": {
                "kind": "float64"
            }
        },
        "signal": {
            "dtype": {
                "kind": "float64"
            }
        }
    },
    "realization": {
        "append_only": true,
        "record_id_column": "__ds_record_id",
        "index_realization": {
            "kind": "none"
        },
        "duplicate_resolution_default": {
            "kind": "latest_by_record_id"
        }
    },
    "compatibility": {
        "allow_inference": true
    }
}
```

This is the baseline for datasets created without `columns=`, `scan=`, or
explicit `logical_indices=`.

Notes:

- `columns` mirrors the frozen payload schema plus Fricon-owned system fields.
  Dtypes are Fricon dataset dtypes. Primitive scalar variants should use the
  same concrete names as Arrow where there is a direct one-to-one mapping, such
  as `float64`, `int64`, and `uint64`. Avoid arbitrary bit-width fields in the
  durable JSON. Fricon-owned business dtypes such as `trace[float64]` and
  `timestamp_us` can extend that vocabulary when they carry semantic meaning.
- Arrow payload schema remains the physical schema. The manifest is the
  semantic authority. New semantic datasets should not rely on Arrow extension
  types to identify Fricon concepts.
- `scan_plan` is absent until the user passes `scan=` or the system later
  materializes inferred scan metadata.
- `index_realization.kind = "none"` means no durable logical index space has
  been declared yet; charts may use compatibility inference if needed.
- `live_defaults` and `view_defaults` are absent until explicitly provided or
  cached by a future implementation.
- Adding optional sections later should not require rewriting payload chunks.

The manifest is not a user-authored JSON format. Design it for Rust serde
maintenance:

- prefer internally tagged enums such as `{"kind": "none"}` over bare strings
  or booleans when a value may later gain fields
- pin durable JSON names explicitly instead of depending on broad serde rename
  rules, especially for acronym-heavy dtype variants
- keep optional top-level sections as `Option<T>` with serde defaults
- avoid `untagged` enums for durable manifest fields
- do not use `deny_unknown_fields` unless intentionally rejecting newer
  manifests
- keep invariant checks in explicit validation code after deserialization

### Column Metadata Complements Data Types

Do not introduce a broad separate column-role system in v1. Most remaining
role-like concepts are better expressed by dataset data types plus small display
or chart hints.

This proposal should preserve Fricon's existing data-type distinctions:

- scalar columns
- complex scalar columns
- trace columns

V1 may extend or refine the Fricon datatype vocabulary when a distinction has
direct storage or rendering consequences, for example:

- timestamp-like scalar columns, if user-provided timestamps need first-class
  rendering or formatting
- categorical scalar columns, if categorical handling differs from plain string
  or numeric values

System timestamps should use `__ds_recorded_at` with the Fricon `timestamp_us`
business dtype when needed. A normal user timestamp column should just be a
typed payload column. The manifest should not expose Arrow timestamp internals.

### Plain Arrow Storage, Manifest Semantics

With a dataset manifest, Arrow extension types become redundant. New semantic
datasets should store plain Arrow physical schemas and put all Fricon-specific
meaning in `dataset_manifest.json`.

Recommended physical layouts:

- `complex128`: `struct<real: float64, imag: float64>`
- simple trace: `list<value>`, with an implicit integer sample index axis
- fixed-step trace: `struct<x0: axis, step: axis, y: list<value>>`
- variable-step trace: `struct<x: list<axis>, y: list<value>>`

`axis` should be a numeric trace-axis dtype, not hard-coded to `float64`.
`value` should be a scalar trace-value dtype such as `float64` or
`complex128`. Nested traces are not a meaningful dataset value and should not
be representable in the trace value type.

The current project has not reached production use, so this is the right time
to remove `fricon.complex` and `fricon.trace` extension metadata as a durable
storage dependency. Compatibility fallback, where kept, should infer from plain
Arrow field shape rather than treating extension metadata as canonical.

### DatasetDataType Refactor Direction

The current runtime `DatasetDataType` collapses physical details into broad
business categories such as numeric scalar, complex scalar, and trace. That was
simple, but it makes future support for `float32`, `int64`, `uint64`,
timestamps, strings, or booleans awkward.

Do not replace it with an arbitrary `(business_kind, arrow_type)` pair. That
would be easy to serialize but would allow many invalid combinations. Prefer a
constrained Fricon-owned dtype enum that uses Arrow-aligned primitive variants
and structured business variants:

```rust
#[serde(tag = "kind")]
enum DatasetDType {
    #[serde(rename = "float64")]
    Float64,
    #[serde(rename = "float32")]
    Float32,
    #[serde(rename = "int64")]
    Int64,
    #[serde(rename = "uint64")]
    UInt64,
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "utf8")]
    Utf8,
    #[serde(rename = "timestamp_us")]
    TimestampUs,
    #[serde(rename = "complex128")]
    Complex128,
    #[serde(rename = "trace")]
    Trace(TraceDType),
}

struct TraceDType {
    layout: TraceLayout,
    axis: TraceAxisDType,
    value: TraceValueDType,
}

enum TraceAxisDType {
    Float64,
    Float32,
    Int64,
    UInt64,
}

enum TraceValueDType {
    Float64,
    Float32,
    Int64,
    UInt64,
    Complex128,
}
```

Validation should check each dtype against the actual plain Arrow physical
schema, but basic invariants should also be encoded in the Rust types. In
particular, nested traces should be impossible to construct rather than merely
rejected later by validation. Runtime consumers should ask semantic questions
through helper methods such as `is_trace`, `is_complex`, `is_numeric`, and
`is_chart_scalar`, instead of pattern-matching every physical primitive
everywhere.

### Column Metadata And Chart Hints

Column metadata should stay lightweight. Users more naturally describe:

- stored payload columns and their optional types, units, and labels
- scan axes and optional static coordinate values
- framework-owned logical indices when append order is insufficient

The public API should therefore accept column metadata such as:

- `dtype`
- `label`
- `unit`
- `hidden_by_default`
- `chart_axis`

`chart_axis` means "this stored column is a good coordinate-axis candidate in
chart UI." It does not create a separate durable coordinate model. All other
chart defaults should be resolved from datatype, scan axes, index realization,
and compatibility inference.

### Scan Plan

`scan_plan` should be lightweight and permissive. It describes the dataset's
logical index space and optional static coordinate values, not a full planner
state dump.

Recommended fields:

- `axes`
- `traversal`
- `planned_shape`
- `strictness`
- `allows_partial`
- `allows_duplicate_positions`
- `index_realization`

Each axis should describe:

- axis identity
- optional static coordinate values
- mode: `linear`, `list`, `categorical`, `adaptive`, `implicit_index`,
  `unknown`

Scan axes are not necessarily stored Arrow payload columns. For example,
`scan={"x": [1, 2, 3], "y": ["left", "right"]}` declares a logical scan grid
whose coordinate lookup values can live in the manifest. For unbounded or
unknown-length axes such as minimizer steps, use an implicit integer index axis
rather than a static list.

If a dataset also stores user columns such as `gate_v` or `field_t`, those
columns remain ordinary payload columns. The scan axis and the payload column
may share a practical relationship, but v1 should not require a one-to-one
mapping between them.

Do not add a separate derived-coordinate model in v1. Scan axes and chart
coordinates can be many-to-many, and derived coordinates are often transforms
of multiple logical indices. If a derived coordinate must be visible after
reopen or export, materialize it as an ordinary stored column and mark it as
usable for chart axes with column metadata such as `chart_axis=True`.

### Logical Index Realization

Logical indices should have three realization modes:

- `implicit`: derive indices from `__ds_record_id`, scan shape, and traversal
- `sidecar`: store append-only index chunks mapping `__ds_record_id` to logical
  index values
- `embedded`: reserve for rare future cases where indices are part of the main
  payload schema

V1 should implement `implicit` and `sidecar`.

Regular ordered scans should default to `implicit`. A scan like:

```python
scan={"gate": [-0.2, -0.1, 0.0, 0.1], "bias": [0.0, 0.01, 0.02]}
```

can derive logical indices from append order and traversal. No per-row index
columns are needed in the main payload.

When append order is insufficient, for example shuffled acquisition, retries,
resumes, adaptive scans, or ragged groups, the writer can persist a sidecar
index table. The sidecar schema should be compact:

```text
__ds_record_id: uint64
<axis_id>: int64
...
```

Sidecar chunks remain append-only and use the same chunking discipline as the
payload. They are dataset facts, but they stay separate from user payload
columns.

### Realization

`realization` should describe dataset facts that affect interpretation, not
mutable summaries that are easy to let drift.

Recommended v1 fields:

- `append_only: true`
- `record_id_column`
- `index_realization`
- `duplicate_resolution_default`

Avoid storing counters such as `actual_points` in the manifest unless there is
clear write ownership and consistency logic.

### Live Grouping

Live grouping should be explicit in the active write session, but it does not
need to be persisted as dataset payload in v1.

Recommended v1 behavior:

- live refresh grouping may use in-memory write-session state
- chart projections after reopen should depend on persisted facts:
  `__ds_record_id`, manifest scan axes, and optional sidecar logical indices
- adjacency-based grouping remains valid only for contiguous legacy scans
- non-contiguous scans that must be reproducible after reopen should persist
  logical indices in the sidecar, not live-only frame markers

Do not persist `sweep_id` or `frame_id` by default in v1. They mainly control
when live charts refresh, and durable execution segments belong more naturally
to a future run or measurement layer.

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
- logical index realization
- value columns
- chart-axis candidate columns
- column display metadata
- live grouping rules
- default views
- duplicate resolution policy

This becomes the API consumed by UI query code and chart transforms.

### Charts Consume Resolved Semantics

Chart selection should stop depending on the coarse `isIndex` bit as its
primary abstraction.

Instead, `crates/fricon-ui` should receive richer dataset interpretation data,
for example:

- `isLogicalIndex`
- `isChartAxisCandidate`
- `axisId`
- `hiddenByDefault`
- `label`
- `unit`
- resolved scan axes
- resolved live grouping defaults
- resolved default views

The current `isIndex` field may exist temporarily as a compatibility projection,
but it should not remain the canonical contract.

## Writer Architecture

### Creation Phase

Dataset creation should support progressive optional metadata while preserving
the simple `write(col=...)` path.

Public Python API shape:

```python
with manager.create(
    "name",
    columns={
        "signal": float,
        "phase": Column(float, unit="rad"),
        "trace": Trace[float],
    },
    scan={
        "gate": [-0.2, -0.1, 0.0, 0.1],
        "bias": [0.0, 0.01, 0.02],
        "step": IndexAxis(),
    },
    description="optional",
    tags=["optional"],
) as writer:
    ...
```

All new semantic parameters should be optional. Users can start with no
metadata, then progressively add the information they consider useful.

Recommended top-level creation arguments:

- `columns`: optional stored payload column definitions
- `scan`: optional logical scan-axis definitions
- future `views`: optional dataset-owned default views, if needed

Avoid making normal users construct or edit the raw manifest. A `semantics=`
escape hatch may be useful later for framework authors, but the primary public
API should stay close to the user's mental model.

### Python Typing Goals

The public Python API should have modern, explicit typing in the `.pyi` stubs so
IDEs, AI coding tools, and static checkers can understand common usage.

Recommended typing direction:

```python
@dataclass(frozen=True)
class Column:
    dtype: ColumnDType | None = None
    unit: str | None = None
    label: str | None = None
    chart_axis: bool | None = None
    hidden_by_default: bool = False

@dataclass(frozen=True)
class IndexAxis:
    label: str | None = None

ColumnSpec = type[float] | type[int] | TraceType | Column
ScanAxisSpec = Sequence[ScalarValue] | IndexAxis | None
```

The runtime may accept plain mappings for convenience, but the documented API
should prefer typed helpers for completion and readability.

Column definitions should support both concise and typed forms:

```python
with manager.create(
    "typed_dataset",
    columns={
        "a": float,
        "b": int,
        "c": Trace[float],
        "v": Column(float, unit="V", label="Voltage"),
    },
) as writer:
    ...
```

If a column definition omits `dtype`, the type is inferred from the first
written row. If no column metadata is supplied, the first row still determines
the user payload schema.

For bare writes:

- Fricon still allows `write(col=...)`
- the first row may still seed user columns for convenience
- but the writer, not the chart layer, is responsible for constructing a
  minimal semantic manifest

### Write Phase

Each row writes:

- user fields
- `__ds_record_id`, materialized automatically
- optional logical index entries through a sidecar when append-order inference
  is insufficient

Keep the high-friction system path out of `write(**kwargs)`. The public
`write()` method should remain a clean row-writing API:

```python
writer.write(x=0.1, y=2.0, signal=0.95)
```

Framework and advanced callers should use `write_dict` for structured write
context:

```python
writer.write_dict(
    {"gate_v": gate_v, "bias_v": bias_v, "signal_v": signal_v},
    logical_indices={"gate": gate_index, "bias": bias_index},
)
```

`logical_indices` is durable dataset context. Supplying it should switch the
dataset's index realization to a sidecar table if implicit realization is not
already sufficient. Do not introduce `status`, `sweep_id`, or `frame_id` as v1
write parameters unless a future run/execution model gives them durable
meaning.

### Transport Creation Contract

Arrow schema should continue to be carried by the IPC stream payload. Creation
metadata should carry only dataset catalog metadata plus optional semantic
creation metadata such as `columns` and `scan`.

This avoids duplicating schema ownership between `CreateMetadata` and the Arrow
payload. If semantic creation metadata changes the protobuf request shape,
`IPC_PROTOCOL_VERSION` should be bumped.

## Reader Architecture

The reader should expose both facts and resolved meaning.

Suggested responsibilities:

- `reader.points()` -> raw row access
- `reader.manifest()` -> raw manifest
- `reader.interpret()` -> resolved dataset semantics
- `reader.indexes()` -> resolved logical indices, derived or sidecar-backed
- `reader.grid(value=...)` -> grid projection using resolved interpretation
- `reader.select(index={...})` -> selection in logical index coordinates
- `reader.live_groups(k=...)` -> live grouping using active-session semantics

Important rule:

- duplicate grid placement must be resolved in the interpretation layer, not
  separately reimplemented in each chart path
- ragged grids should be represented as observed sparse index pairs rather than
  forced into rectangular planned shapes

## Duplicate Semantics

Duplicate logical positions are allowed and expected.

Recommended v1 rule:

- default duplicate policy: `latest_by_record_id`

Do not add status-aware duplicate policies in v1. Dataset payloads are
append-only facts; when the same logical scan point is written more than once,
the default projection should use the later `__ds_record_id`. If a future
workflow needs invalidation or execution-quality state, it should be designed
with the run or measurement layer rather than added as an under-specified
dataset column.

## Live Monitor Rules

### Contiguous Legacy Scans

Allowed fallback:

- derive sweeps from slow-axis tuple changes
- derive live heatmap frame from the last contiguous outer group

### Semantic Datasets

Preferred path:

- use manifest scan axes and resolved logical indices
- use in-memory live grouping for active write-session refresh timing
- use manifest-configured fallback grouping only when scans are contiguous

### Non-Contiguous Scans

For shuffled scans, resumes, ROI-first acquisition, or retries:

- explicit logical indices are required for durable correctness
- adjacency heuristics are not sufficient
- the durable representation should be the logical-index sidecar, not
  `sweep_id` or `frame_id` payload columns

This should be documented as a hard semantic contract, not a best-effort hope.

### Unknown-Length And Ragged Scans

Unknown-length axes are first-class in v1 for minimizer and adaptive workflows.
The common minimizer case should be modeled as an implicit integer index:

```python
scan={"step": IndexAxis()}
```

or a concise equivalent chosen by the Python API.

This axis has no planned length and no static coordinate list. By default,
`step` is derived from append order. If a workflow has multiple ragged groups,
for example restarts with different minimizer lengths, it should persist a
sidecar logical index table:

```text
__ds_record_id | restart | step
0             | 0       | 0
1             | 0       | 1
2             | 0       | 2
3             | 1       | 0
4             | 1       | 1
```

Charts should treat this as an observed sparse grid. Missing cells are empty;
the dataset does not need to claim a rectangular `planned_shape`.

## Feature Shaping

The architecture should land as shaped feature layers, not as one broad v1.
This keeps the current route dataset-first while leaving clean attachment
points for later experiment, parameter, provenance, workflow, AI, and device
models.

### Feature 1: Durable Dataset Semantics Foundation

Classification: `now`.

User value:

- bare Python writes keep working
- every new dataset gets durable local meaning
- old datasets and fixtures can still open through compatibility inference
- future run, parameter, provenance, and workflow models get a stable dataset
  anchor without being hidden inside chart heuristics

Scope:

- chunked append-only Arrow payloads remain the storage substrate
- `dataset_manifest.json`
- `__ds_record_id`
- reserved `__ds_` system-field prefix
- minimal manifest for bare writes with column metadata, realization, and
  compatibility settings
- manifest IO, serde model, validation, and defaulting
- plain Arrow physical schemas for new semantic datasets
- compatibility fallback for datasets without manifests
- resolved interpretation API for readers and downstream consumers
- temporary adapters that preserve existing chart and dataset-detail behavior

Out of scope:

- new public `columns=` metadata
- public `scan=`
- logical-index sidecar chunks
- full chart/live-view rewrite
- status, invalidation, or quality-state semantics
- run, measurement, parameter, provenance, workflow, AI, or device manifests

First success criterion:

- new datasets produce a valid manifest and record IDs while existing datasets
  still open and current user-facing behavior remains equivalent.

ADR need:

- create an ADR before implementation commits to the durable manifest shape,
  record-id semantics, compatibility policy, and interpretation-layer boundary.

### Feature 2: Progressive Column Metadata API

Classification: `next`, after the foundation can persist and resolve manifest
column metadata.

User value:

- users can add units, labels, hidden-by-default state, and chart-axis hints
  without learning scan-plan or manifest vocabulary
- future parameter and provenance views get consistent display metadata without
  treating column names as the only source of meaning

Scope:

- optional `columns=` creation metadata
- typed Python helpers such as `Column`
- declared dtypes where useful, while still allowing first-row inference when
  omitted
- `unit`, `label`, `hidden_by_default`, and `chart_axis` metadata
- reader and UI detail surfaces expose resolved column metadata

Out of scope:

- `scan=`
- logical-index sidecar chunks
- parameter schemas or parameter-set versioning
- user-authored raw manifests

### Feature 3: Explicit Scan Semantics

Classification: `next`.

User value:

- regular ordered scans can be reopened without row-adjacency guesses
- non-contiguous, resumed, adaptive, and ragged acquisition can be represented
  durably
- framework-owned experiment systems can provide exact logical indices without
  burdening simple Python scripts

Scope:

- optional `scan=` creation metadata
- implicit logical-index realization for regular ordered scans
- unknown-length integer index axes for minimizer-style workflows
- optional append-only logical-index sidecar chunks
- duplicate projection using `latest_by_record_id`
- sparse/ragged grid projection from observed index pairs
- active-session live grouping that does not require durable `sweep_id` or
  `frame_id` payload columns

Out of scope:

- execution segment tables
- status-aware duplicate policies
- expanded planned-point tables
- a separate durable derived-coordinate model
- null-heavy or late-appearing column workflows

### Feature 4: Consumer Migration

Classification: `next`, after the foundation is stable.

User value:

- desktop detail, chart defaults, heatmaps, and live views consume resolved
  dataset interpretation instead of rediscovering semantics from raw rows

Scope:

- dataset detail DTOs expose resolved semantic fields such as logical index,
  chart-axis candidate, label, unit, scan axes, and defaults
- chart transforms consume resolved logical indices and duplicate policy
- `isIndex` remains temporarily as a compatibility projection
- legacy direct use of index-column inference is removed after migration

Out of scope:

- turning the desktop UI into the primary experiment execution engine
- saved user-owned chart presets or workspace UI state inside the dataset
  manifest

### Deferred Concepts

The dataset semantic layer should leave room for these concepts but not
implement them:

- measurement or run manifest design
- execution segment tables
- status or invalidation semantics
- rich provenance graph
- parameter snapshots and diffs
- workflow definitions and workflow runs
- AI action approval or audit records
- device identity and configuration snapshots
- user-editable manifest history
- null-heavy or late-appearing column workflows

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
2. Fricon materializes `__ds_record_id` automatically.
3. Fricon writes a minimal manifest with:
    - observed payload column metadata
    - `__ds_record_id` as the record-id system field
    - no scan plan
    - no durable logical index realization
    - compatibility mode enabled
4. Readers and charts may still use compatibility inference for this dataset.

Result:

- the user gets the same "just start writing rows" experience as today
- the dataset is still owned by the new architecture
- no user-visible semantic burden is introduced for the simple path

Optional upgrade path:

If the user wants slightly better semantics without much extra ceremony, the
API should allow top-level `columns` and `scan` metadata:

```python
with ws.dataset_manager.create(
    "quick_iv_map",
    columns={
        "gate_v": Column(float, unit="V"),
        "bias_v": Column(float, unit="V"),
        "current_a": Column(float, unit="A"),
    },
    scan={
        "gate": [-0.2, -0.1, 0.0, 0.1, 0.2],
        "bias": [0.0, 0.01, 0.02],
    },
) as ds:
    for ix, gate_v in enumerate([-0.2, -0.1, 0.0, 0.1, 0.2]):
        for bias_idx, bias_v in enumerate([0.0, 0.01, 0.02]):
            ds.write(gate_v=gate_v, bias_v=bias_v, current_a=measure_current())
```

This example combines the column-metadata and scan-semantics features. Column
metadata can land first without requiring scan declarations or sidecars.

The `scan` declaration communicates logical axes without asking the user to
learn role vocabulary. For this regular ordered scan, Fricon can derive logical
indices from append order and scan traversal.

For a minimizer or adaptive workflow, the user can declare an unknown-length
integer index axis:

```python
with ws.dataset_manager.create(
    "minimize",
    columns={"loss": float, "x": float, "y": float},
    scan={"step": IndexAxis()},
) as ds:
    for step in optimizer:
        ds.write(loss=step.loss, x=step.x, y=step.y)
```

This keeps the semantic path available but not mandatory.

### Use Case 2: Fully Wired Experiment System

Scenario:

- the user works inside a higher-level experiment system
- scan configuration, loop structure, planner state, and system columns are
  wired by the experiment runtime
- the experiment author should focus on dataset intent and the loop body, not
  on dataset plumbing

Recommended split of responsibility:

- the experiment system owns:
    - scan specification
    - traversal policy
    - retry or resume policy
    - explicit logical indices when append order is insufficient
    - live grouping state for active refresh timing
    - loop orchestration
- the experiment author owns:
    - dataset declarations that matter to the experiment
    - the loop body function or measurement function

Example shape:

```python
@experiment(
    dataset={
        "name": "transport_map",
        "columns": {
            "gate_v": {"dtype": float, "unit": "V"},
            "field_t": {"dtype": float, "unit": "T"},
            "signal_v": {"dtype": float, "unit": "V"},
        },
    },
    scan={
        "gate": gate_points,
        "field": field_points,
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
2. It decides whether logical indices are implicit or sidecar-backed:
    - regular ordered scans can use implicit realization
    - shuffled, resumed, adaptive, or ragged scans should use sidecar
      realization
3. It creates the manifest before the first data row is written.
4. Each loop iteration writes one payload row, plus sidecar logical indices
   when needed.

The actual write path for a non-contiguous or ragged scan may look like:

```python
ds.write_dict(
    {
        "gate_v": ctx.axis("gate").value,
        "field_t": ctx.axis("field").value,
        "signal_v": lockin_read(),
    },
    logical_indices={
        "gate": ctx.axis("gate").index,
        "field": ctx.axis("field").index,
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
- the experiment author only writes dataset intent plus measurement logic

## API Design Implication

The public API should intentionally support two entry modes:

- minimal mode:
    - optimized for ad hoc Python scripts
    - zero or near-zero semantic boilerplate
    - compatibility inference remains acceptable
- semantic mode:
    - optimized for framework-owned execution
    - columns and scan declared before acquisition starts where possible
    - fixed payload schema, explicit logical indices when needed, explicit scan
      plan

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
2. Update dataset creation to produce manifests for all new datasets.
3. Keep a compatibility inference path only for old fixtures and ad hoc test
   data.
4. Switch chart and live-query code to consume resolved interpretation after
   the foundation is stable.
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

The foundation feature is considered successful when:

- bare `write(col=...)` still works, but now produces a minimal semantic
  manifest owned by the dataset layer
- every new semantic dataset has stable `__ds_record_id` values
- old datasets without manifests still open through compatibility inference
- readers can expose raw rows, raw manifest data, and resolved interpretation
- current chart and dataset-detail behavior remains equivalent through
  compatibility adapters

The later column, scan, and consumer-migration features are considered
successful when:

- declared column units, labels, hidden state, and chart-axis hints round trip
  through creation, manifest storage, reader interpretation, and dataset detail
  surfaces
- a known 2D scan opens correctly from explicit semantics without relying on
  first-two-row inference
- regular ordered scans can derive logical indices implicitly without storing
  index columns in the main payload
- shuffled scans with sidecar logical indices render to the correct grid cells
- retries and resumes preserve all fact rows while default projections use the
  latest row by `__ds_record_id`
- live views for active writes can use in-memory grouping without requiring
  durable `sweep_id` or `frame_id` payload columns
- minimizer-style unknown-length index axes can be sliced by index coordinate
- ragged heatmaps render from observed sparse index pairs

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
