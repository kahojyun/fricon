# ADR 0002: Decide Dataset Semantic Manifest V1

## Status

Accepted

## Context

Fricon's current dataset stack stores append-only Arrow chunk files and derives
chart meaning from row ordering and coarse index-column inference. The dataset
semantics proposal defines a replacement architecture where durable dataset
meaning lives beside the payload in a manifest and consumers depend on resolved
semantics instead of rediscovering interpretation from raw rows.

Implementation should not start until the durable foundation is settled. The
first decision must cover the v1 manifest shape, system record IDs, reserved
system fields, dtype vocabulary, Arrow storage direction, minimal axis
inference, versioning implications, and the boundary between storage,
semantics, interpretation, and UI or chart consumers.

## Decision

New semantic datasets will have a dataset-local sidecar named
`dataset_manifest.json`. The v1 manifest is a Fricon-owned durable JSON format,
not a user-authored public configuration file.

The minimal v1 manifest has these top-level sections:

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
    "inference": {
        "allow_axis_inference": true
    }
}
```

`manifest_version`, `columns`, `realization`, and `inference` are required
for v1. Future or non-minimal semantic datasets may add optional sections such
as `scan_plan`, `live_defaults`, and `view_defaults`, but those sections are not
part of the foundation implementation.

Manifest enums should use internally tagged JSON objects such as
`{"kind": "none"}` rather than bare strings when a value may later need fields.
Rust serde definitions should pin durable JSON names explicitly instead of
depending on broad rename rules, especially for acronym-heavy dtype variants.
Rust serde structs should keep optional sections defaultable and put invariant
checks in explicit validation after deserialization. The durable format should
not use `untagged` enums or broad `deny_unknown_fields` defaults.

Every new semantic dataset will materialize a system column:

```text
__ds_record_id: uint64
```

`__ds_record_id` is monotonically increasing within a dataset, stable across
restart, archive, export, and import, and defines append order. When duplicate
logical positions exist, the default projection chooses the row with the latest
record ID. The record ID is part of raw dataset facts and the Arrow payload, but
it is hidden by default from user-facing reads, table views, chart defaults, and
ordinary Python conveniences. Explicit raw/facts APIs may expose it.

The prefix `__ds_` is reserved for Fricon-owned system fields. User payload
columns using this prefix are rejected at dataset creation or first-row schema
freeze time. Existing manifest-free datasets are intentionally unsupported by
the semantic reader in this PR and are not migrated.

The v1 dtype vocabulary is a constrained Fricon-owned dataset dtype enum with
Arrow-aligned primitive names and structured business variants. The initial
vocabulary is:

- `float64`
- `float32`
- `int64`
- `uint64`
- `bool`
- `utf8`
- `timestamp_us`
- `complex128`
- `trace`

Trace dtypes carry structured layout, axis dtype, and value dtype fields.
Allowed trace axis dtypes are numeric scalar dtypes. Allowed trace value dtypes
are numeric scalar dtypes and `complex128`. Nested traces are invalid by type,
not merely by late validation.

New semantic datasets should use plain Arrow physical schemas. Fricon-specific
meaning belongs in `dataset_manifest.json`, not in Arrow extension metadata.
The intended physical mappings are:

- `complex128` as `struct<real: float64, imag: float64>`
- simple trace as `list<value>` with an implicit `uint64` sample index axis
- fixed-step trace as `struct<x0: axis, step: axis, y: list<value>>`
- variable-step trace as `struct<x: list<axis>, y: list<value>>`

Minimal axis inference remains available for simple semantic manifests without
scan plans or explicit column metadata. It preserves the low-friction
`write(col=...)` path while still requiring a manifest. Manifest-free datasets,
old fixtures, and exploratory raw Arrow directories are outside this PR's
compatibility scope.

The compatibility and versioning decisions for the foundation are:

- adding `dataset_manifest.json` and `__ds_record_id` changes durable dataset
  payload rules and must follow the dataset payload layout checklist
- no old-workspace reader compatibility is preserved for manifest-free dataset
  payloads in this PR
- adding semantic creation metadata to Rust IPC requests requires an
  `IPC_PROTOCOL_VERSION` decision; keeping schema in the Arrow stream and
  adding no required wire fields does not by itself require an IPC bump
- archive export/import must either include the manifest and any future
  semantic sidecars or deliberately document legacy archive behavior; required
  archive metadata changes require an archive version decision
- public user-facing behavior changes need normal release-note consideration

The ownership boundary is:

- dataset storage owns Arrow chunk IO, dataset file layout, and payload facts
- dataset semantics owns manifest serde types, defaulting, validation, and
  minimal axis inference settings
- dataset interpretation owns resolved meaning, logical indices, duplicate
  projection, and chart-ready questions
- UI, chart, and live-view consumers ask interpretation APIs for meaning
  instead of inferring it from field order, row adjacency, or legacy `isIndex`

Higher-level run, measurement, parameter, provenance, workflow, AI, and device
manifests remain out of scope for the dataset manifest v1.

## Consequences

Implementation can start with a narrow foundation: manifest IO, validation,
record ID materialization, plain Arrow physical schemas for new datasets,
minimal semantic inference for simple datasets, and a resolved interpretation
API.

Bare Python writes remain low-friction. Users do not need to author raw
manifests, declare scan plans, or understand record IDs for simple datasets.

Chart and UI behavior may be preserved temporarily through adapters, but the
long-term contract is resolved interpretation rather than legacy index-column
inference.

The manifest becomes a compatibility surface. Future changes to required
manifest fields, system-field semantics, payload schema rules, archive entries,
workspace layout, or IPC creation metadata must use the relevant maintenance
and versioning checklists.

## Alternatives Considered

Keeping semantics in Arrow extension metadata would avoid a sidecar file, but
it would keep Fricon-specific meaning inside the physical storage schema and
make interpretation harder to version independently.

Keeping `__ds_record_id` virtual would avoid one physical column, but it would
make append order less portable across restart, export, import, sidecar logical
indices, and duplicate projection.

Exposing `__ds_record_id` in ordinary user reads would make all stored facts
visible by default, but it would leak internal plumbing into the simple Python
and desktop workflows. Explicit raw/facts APIs are a better fit for callers
that need system fields.

Allowing user columns with the `__ds_` prefix would maximize naming freedom,
but it would make future system fields ambiguous and force fragile collision
handling after datasets already exist.

Continuing to infer chart semantics as the primary path would preserve current
behavior with less implementation work, but it would keep the wrong boundary in
place and block durable support for non-contiguous scans, retries, resumes,
ragged grids, and explicit scan semantics.
