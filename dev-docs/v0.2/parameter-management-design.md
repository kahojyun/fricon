# Parameter Management Design Proposal

## Status

Supporting future parameter-management proposal for the v0.2 reset.

This is not current behavior. Do not implement or document behavior from this
proposal as user-facing functionality until the relevant dataset, run,
provenance, storage, IPC, Python API, and desktop UI work has landed.

Read `README.md` and `design.md` first. This long proposal is supporting
detail for parameter snapshots, profiles, diffs, proposals, and calibration
integration; it is not the canonical v0.2 entry point.

## Purpose

Define a long-term parameter management direction for Fricon that fits the
local-first, Python-led scientific workflow route.

The proposed system is a versioned parameter registry for long-lived scientific
experiment and numerical simulation parameters. It should help users answer:

- Which parameter state was used for this run?
- Which mutable profile or ref did that state come from?
- What changed between two parameter states?
- Why did a parameter value change?
- Which dataset, analysis step, or calibration proposal produced the change?
- Which runs may be affected by an outdated or incorrect parameter value?

The design should support measurement experiments, calibration workflows, and
numerical simulation workflows without turning Fricon into a distributed version
control system, a hardware driver framework, or a hosted lab information
system.

## Classification

Classification: `after dataset semantics`.

ADR need: create an ADR before implementation commits to the durable parameter
model, storage shape, Python API contract, desktop UI semantics, or run
provenance integration.

This proposal depends on the current dataset-semantic baseline and the proposed
v0.2 data-library reset:

- dataset semantics should land before higher-level parameter behavior depends
  on datasets as provenance anchors
- Python-led run records should exist before parameter snapshots become run
  inputs
- parameter concepts should not be hidden in dataset names, incidental dataset
  metadata, or chart heuristics
- direct hardware application belongs to a later device or execution subsystem,
  not to the parameter registry itself

## Product Goals

### Reproducibility

Every run should be able to record the exact immutable parameter snapshot used
at run start. If a user starts from a mutable ref such as `main`, Fricon should
resolve it before execution and record the resolved snapshot ID.

Recording only a mutable ref is insufficient because refs can move.

### Traceability

Fricon should let users inspect the chain from parameter state to run, dataset,
analysis result, parameter update proposal, and later parameter snapshot.

The goal is not only to replay a run, but also to explain why important
scientific values changed.

### Safe Parameter Evolution

Scientific parameter structure changes over time. Users may add, delete, move,
rename, or change parameter types as devices, samples, calibration methods, and
simulation models evolve.

The system should treat schema changes as first-class parameter operations
rather than reducing them to opaque JSON replacement.

### Scientific Structure

The model should support two common shapes:

- one optional tree section for sparse, irregular, nested parameters
- zero or more table sections for regular parameter families such as gate
  calibration tables, coupling maps, solver coefficients, or boundary
  conditions

### Local-First Simplicity

The parameter system should work well for one researcher using local Python
scripts and the desktop UI. Multi-user collaboration, permissions, distributed
merge semantics, and hosted service behavior are explicit non-goals for the
current product route.

## Non-Goals

The first parameter management design should not include:

- full Git-like merge semantics
- multi-user review or access control
- automatic direct apply-to-hardware from a diff view
- broad instrument driver or orchestration APIs
- first-class ndarray storage semantics
- arbitrary computed parameter expression languages
- hiding parameter state inside dataset metadata
- making the desktop UI the primary execution engine

## Core User Model

The parameter system has two audiences:

- direct users who want to manage scientific parameters through Python or the
  desktop UI
- other Fricon systems that need stable parameter bindings, proposals, and
  validation results

Keep those audiences separate in the terminology. The public user model should
stay small, while integration and internal terms can be more precise.

### Public Registry Terms

| Concept            | Meaning                                                                           |
| ------------------ | --------------------------------------------------------------------------------- |
| Parameter registry | Workspace-local store of parameter snapshots, refs, tags, drafts, and history.    |
| Parameter snapshot | Immutable full parameter state plus schema at one point in history.               |
| Profile / ref      | Mutable pointer to a snapshot, such as `main`, `dev`, or `cooldown/2026-04/main`. |
| Tree section       | Optional nested parameter structure for sparse and irregular values.              |
| Table section      | Named keyed table for regular parameter families.                                 |
| Schema             | Types, units, constraints, table keys, lifecycle status, and display metadata.    |
| Tag                | Stable human alias for an important snapshot.                                     |
| Draft              | Editable working state based on a snapshot or import.                             |
| Patch              | Structured set of tree, table, schema, or metadata changes.                       |
| Diff               | Comparison between snapshots, drafts, or selected scopes.                         |
| History            | Value, schema, ref, and proposal history.                                         |

Avoid using "parameter set" for every nested object. In Fricon terminology, a
single parameter snapshot should contain sections. Each section may be tree-like
or table-like, but the snapshot is the unit that a run freezes.

The eventual public API should choose one primary term for mutable pointers.
`Profile` is friendlier for users, while `ref` is shorter and maps well to
storage and integration APIs. Avoid exposing `branch` as the primary term unless
the implementation intentionally supports Git-like expectations.

### Integration Terms

These concepts are needed when other systems connect to the registry, but they
should not make the registry responsible for execution, analysis, or device
control.

| Concept              | Meaning                                                                    |
| -------------------- | -------------------------------------------------------------------------- |
| Parameter binding    | Result of resolving a profile/ref to an immutable snapshot ID and hash.    |
| Run-local parameters | Inputs specific to one run, owned by the run or experiment system.         |
| Runtime overrides    | Run-scoped temporary patch against a resolved parameter snapshot.          |
| Effective run config | Run/execution-owned configuration derived from parameters and run inputs.  |
| Parameter proposal   | Reviewable patch proposed by analysis, calibration, import, or automation. |
| Validation report    | Structured result from checking drafts, proposals, overrides, or commits.  |

Parameter management resolves and versions parameter state. Other systems
consume, bind, propose changes to, or derive execution configs from that state.

### Internal Identity Terms

Stable IDs and tombstones support history and refactor-aware diff. They should
exist in storage and advanced APIs, but they are not the default user mental
model.

| Concept       | Meaning                                                           |
| ------------- | ----------------------------------------------------------------- |
| `param_id`    | Stable identity for a tree leaf.                                  |
| `table_id`    | Stable identity for a table section.                              |
| `column_id`   | Stable identity for a table column.                               |
| `row_id`      | Optional identity for rows that survive primary-key changes.      |
| `tombstone`   | Logical deletion marker that preserves historical interpretation. |
| snapshot hash | Integrity and reproducibility hash for a materialized snapshot.   |

## Parameter Snapshot Shape

A parameter snapshot is an immutable bundle:

```text
ParameterSnapshot
  snapshot_id: p_8f31c2a
  parent_ids: [p_12ab901]
  source_ref: main
  message: "retune q0/q1 gates"
  context:
    sample: sample-17
    cooldown: 2026-04
  tree:
    ...
  tables:
    one_qubit_gates:
      ...
    two_qubit_gates:
      ...
```

The same shape can describe measurement and simulation use cases.

Measurement-oriented example:

```text
tree:
  experiment:
    repetitions: 1000
    cooldown_s: 0.2
  devices:
    awg:
      sample_rate_hz: 2400000000.0

tables:
  one_qubit_gates:
    key: [qubit, gate]
    columns: amplitude, duration_ns, phase_rad
  two_qubit_gates:
    key: [control, target, gate]
    columns: amplitude, duration_ns, detuning_hz
```

Simulation-oriented example:

```text
tree:
  solver:
    method: rk45
    tolerance: 1e-9
    max_steps: 100000
  random:
    seed: 12345

tables:
  model_coefficients:
    key: [component, term]
    columns: value, unit
  initial_conditions:
    key: [state]
    columns: value
  boundary_conditions:
    key: [boundary, variable]
    columns: kind, value
```

Large arrays, meshes, waveforms, and simulation grids should initially stay out
of first-class parameter values. Store references plus small metadata in the
parameter snapshot and keep the large payload in datasets or later artifact
storage:

```text
tree:
  model:
    mesh_dataset: ds_42
    initial_state_dataset: ds_57
```

## Tree Section

The tree section is suitable for sparse and heterogeneous values:

```text
/devices/vna/init/if_bandwidth
/devices/vna/init/power_dbm
/devices/awg/channels/ch1/default_offset
/experiment/defaults/readout_repetitions
/solver/tolerance
```

Each leaf should have schema metadata:

```json
{
    "param_id": "param_001",
    "path": "/devices/vna/init/if_bandwidth",
    "dtype": "float64",
    "unit": "Hz",
    "nullable": false,
    "description": "VNA IF bandwidth used during initialization",
    "constraints": {
        "min": 1.0,
        "max": 1000000.0
    },
    "value": 1000.0
}
```

Paths are addresses, not identity. A rename or subtree move should preserve the
stable parameter ID so history can follow the scientific concept.

## Table Sections

Table sections are suitable for structured parameter families:

```text
table: one_qubit_gates
primary_key: [qubit, gate]

columns:
  qubit: string
  gate: enum[X90, X180, Y90, Y180]
  amplitude: float64
  duration_ns: float64
  phase_rad: float64
  drag_beta: nullable float64
```

Diff and history should use table keys and column identities, not row numbers.

Example table diff:

```text
one_qubit_gates[qubit=q3, gate=x90].amplitude:
  0.123 -> 0.127

two_qubit_gates[control=q1, target=q2, gate=cz].duration_ns:
  added 42.0
```

For ndarray-like structures, use table layout plus optional view hints rather
than first-class array semantics:

```text
table: readout_crosstalk_matrix
primary_key: [row_qubit, col_qubit]
columns:
  row_qubit: string
  col_qubit: string
  coupling: float64
view_hint:
  kind: matrix
  row_axis: row_qubit
  col_axis: col_qubit
  value: coupling
```

MVP behavior may leave matrix reconstruction and visualization to user scripts.

## Identity And Addressing

User-visible names should be editable. Durable identity should be stable.

Recommended minimal identity model:

```text
Required:
  param_id
  table_id
  column_id

Optional:
  row_id
  entity_id

Usually unnecessary:
  cell_id
  value_id
```

Tree values can be addressed by:

```text
snapshot_id + param_id
```

Table cells can be addressed by:

```text
snapshot_id + table_id + row_key + column_id
```

Most table rows should be keyed by explicit primary-key columns. Stable `row_id`
is useful only when row identity survives primary-key changes, such as physical
entity renaming.

## Schema Evolution

Each snapshot should be interpreted with its own schema. Historical snapshots
should not be forced into the latest schema.

Tree schema operations should include:

```text
create_param(path, schema, value)
set_param(path_or_id, value)
rename_param(old_path, new_path)
move_subtree(old_prefix, new_prefix)
deprecate_param(path_or_id)
delete_param(path_or_id)
change_type(path_or_id, new_type, migration?)
```

Table schema operations should include:

```text
create_table(name, primary_key, columns)
rename_table(old_name, new_name)
add_column(table, column, schema, default?)
rename_column(table, old_column, new_column)
change_column_type(table, column, new_type, migration?)
drop_column(table, column)
change_primary_key(table, new_primary_key, migration?)
```

Deletion should be logical, not physical. Tombstones preserve history and allow
old snapshots, diffs, and run records to remain interpretable.

Lifecycle status should be available for parameters, tables, and columns:

```text
active
deprecated
hidden
deleted
archived
```

## Validation

Validation should be layered:

```text
type validation:
  float64, int64, bool, string, enum, nullable, table schema

unit and shape validation:
  Hz, dBm, ns, rad, matrix-like table hints

constraint validation:
  min/max, allowed set, uniqueness, monotonicity

domain validation:
  project-specific checks such as valid gate duration or allowed qubit pair
```

The parameter system should own type, nullable, table-key, and basic constraint
validation. Domain validation should be extensible later through Python hooks
or workflow-owned checks.

Validation outcomes should distinguish:

```text
error:
  block commit or run

warning:
  allow continuation but record explicitly

info:
  UI-level hint
```

## Refs, Profiles, And Tags

Product language should prefer `ref` or `profile` over `branch` in user-facing
surfaces. "Branch" is a useful analogy, but it may imply full Git semantics.

Examples:

```text
main
dev
device/device-a/main
cooldown/2026-04/main
sample/sample-17/main
experiment/rabi/defaults
calibration/rabi/latest
```

Ref updates should use optimistic locking:

```text
update_ref("main", new_snapshot, expected_old_snapshot)
```

This prevents accidental overwrite when two editing or calibration workflows
advance the same ref.

Tags are stable aliases for meaningful snapshots:

```text
cooldown-2026-04-start
before-q3-retune
paper-figure-2-parameters
```

## Run Integration

Runs should record parameter state as resolved immutable facts.

At minimum, a future run record should include:

```json
{
    "parameter_ref": "main",
    "resolved_parameter_snapshot": "p_8f31c2a",
    "resolved_parameter_snapshot_hash": "sha256:...",
    "resolved_at": "2026-04-28T10:30:00Z"
}
```

Run-local parameters should be recorded separately:

```json
{
    "run_parameters": {
        "qubit": "q3",
        "gate": "x90",
        "amp_start": 0.05,
        "amp_stop": 0.25,
        "points": 101,
        "repetitions": 2000
    }
}
```

Runtime overrides should be explicit:

```json
{
    "parameter_overrides": [
        {
            "target": "one_qubit_gates[qubit=q3, gate=x90].amplitude",
            "base_value": 0.123,
            "override_value": 0.13,
            "reason": "temporary debug override"
        }
    ]
}
```

Run-local parameters and runtime overrides are different:

- `amp_start` is a one-run scan input
- overriding `q3/x90` amplitude modifies a long-lived parameter slot for one
  run only

The run may also store an effective configuration hash or snapshot after
combining the resolved parameter snapshot, run-local parameters, runtime
overrides, and derived values:

```json
{
    "effective_config_hash": "sha256:...",
    "compiled_plan_hash": "sha256:...",
    "derived_parameters": {
        "drive_frequency_hz": 4812000000.0,
        "pulse_duration_ns": 20.0,
        "waveform_ref": "artifact:wf_123"
    }
}
```

Instrument snapshots belong to run provenance, not to the parameter registry.
They should reference actual device state, readbacks, firmware, connection
status, or applied setpoints when future device integration exists.

## Analysis-Driven Updates

Parameter updates should usually be proposed after data collection and analysis,
not silently written by experiment scripts during execution.

Recommended flow:

```text
old snapshot
  -> experiment run
  -> measured dataset
  -> analysis run
  -> analysis result
  -> parameter update proposal
  -> draft
  -> validation
  -> new snapshot
  -> optional ref update
```

A proposal should carry enough provenance to explain the change:

```json
{
    "proposal_id": "prop_789",
    "base_parameter_snapshot": "p_001",
    "source_analysis_id": "ana_456",
    "patch": [
        {
            "op": "replace_cell",
            "table": "one_qubit_gates",
            "key": {
                "qubit": "q3",
                "gate": "x90"
            },
            "column": "amplitude",
            "old_value": 0.123,
            "new_value": 0.124
        }
    ],
    "quality": {
        "status": "passed",
        "confidence": 0.91
    }
}
```

Even automated calibration should make parameter mutation explicit. Depending
on future policy, a good result may auto-commit to a calibration profile, while
promotion to `main` may require review.

## Diff And Apply

Diffs should be structured, refactor-aware, and selectively applicable.

Machine-oriented diff example:

```json
[
    {
        "op": "replace",
        "target": "tree",
        "path": "/devices/vna/init/if_bandwidth",
        "old": 1000,
        "new": 3000,
        "dtype": "float64",
        "unit": "Hz"
    },
    {
        "op": "replace_cell",
        "target": "table",
        "table": "one_qubit_gates",
        "key": {
            "qubit": "q3",
            "gate": "x90"
        },
        "column": "amplitude",
        "old": 0.123,
        "new": 0.127
    }
]
```

Human-oriented diff example:

```text
Devices / VNA
  /devices/vna/init/if_bandwidth
    1000 Hz -> 3000 Hz

One-qubit gate table
  q3 / x90 / amplitude
    0.123 -> 0.127
```

Refactor-aware diff should display renames as renames, not as delete plus add:

```text
Renamed parameter:
  /devices/vna/init/ifbw
  -> /devices/vna/init/if_bandwidth

Value unchanged:
  1000 Hz
```

Apply should target drafts:

```text
diff -> selected patch -> draft -> validate -> commit -> optional ref update
```

Do not make direct apply-to-hardware part of the parameter diff MVP. Hardware
application needs validation, dry run, dependency ordering, command compilation,
safety checks, readback verification, partial failure handling, and audit logs.

## UI Direction

A future desktop parameter view could use this structure:

```text
[Profile selector] [Snapshot selector] [Search path/table]

Left:
  tree and table section navigation

Center:
  values, schema metadata, filters, and table views

Right:
  history, diff, provenance, validation, and notes

Bottom:
  draft changes and commit controls
```

Tree UI should support:

```text
path search
changed-only view
schema metadata display
rename and deprecation status
subtree move
history following param_id
```

Table UI should support:

```text
key-based row lookup
changed-row and changed-column filters
selected cell history
sort by absolute or relative delta
selective apply to draft
summary statistics for large diffs
```

The UI should make risk visible. Examples:

```text
low:
  description or display label changed

medium:
  calibration value changed significantly
  nullable changed
  constraint changed

high:
  type changed
  parameter deleted
  primary key changed
  hardware safety-related parameter changed
```

## Import And Export

Fricon should eventually support two export classes.

Machine-complete export preserves:

```text
IDs
schema
metadata
snapshot history
refs and tags
provenance
lifecycle status
```

User-editable export favors:

```text
JSON
YAML
CSV table exports
Python dictionary import
```

Editable imports should be explicit about matching mode:

```text
match by embedded IDs
match by path/name
create new identities
```

## Additional Design Constraints

### Units And Display Values

Units are a high-risk area because many scientific values remain numerically
valid while their interpretation changes. The parameter schema should record a
canonical unit for unit-bearing values, while UI and export layers may expose
display units.

Diffs should compare canonical values and canonical units. A display-only
change such as `1000 Hz` to `1 kHz` should not appear as a scientific value
change if the canonical value is unchanged.

Changing a canonical unit or converting stored values should be an explicit
schema or migration operation, not an incidental display preference.

### Starting From Empty Or Rough Imports

Users should not need to design a perfect schema before using the registry. A
new workspace should be able to start from:

```text
empty draft
JSON or YAML tree import
CSV table import
Python dictionary or list-of-dicts import
manually entered tree paths and table rows
```

The registry may infer initial dtypes and mark them as inferred. Users can
later refine units, constraints, descriptions, lifecycle status, table keys,
and display metadata through normal drafts and commits.

This keeps the standalone parameter system useful before experiment execution,
device management, or workflow automation exists.

### Snapshot Usage In Runs

Run records need enough parameter information to reproduce or explain a run
without making the parameter registry own run execution.

Recommended staged strategy:

```text
MVP:
  source profile/ref
  resolved snapshot ID and hash
  run-local parameters
  runtime overrides
  effective config hash or serialized effective config owned by the run system

Later:
  used-parameter manifest for impact analysis and stale-parameter detection
```

A full resolved parameter snapshot is simple and robust but may include values
unused by a run. A used-parameter manifest is more precise but requires access
tracing or explicit declaration. Treat used-parameter manifests as a later
capability, not an MVP requirement.

### Draft Conflicts

The MVP does not need full Git-like merge, but it should define minimal conflict
semantics for drafts based on the same snapshot.

If two drafts modify the same tree parameter, table cell, table schema, or ref
from the same base, updating the same profile/ref should require explicit
resolution. If two drafts modify distinct slots, an automatic rebase can be a
later convenience.

Conflict resolution should produce an ordinary draft and commit, not a hidden
mutation.

### Structured Context Metadata

Profile names may encode context, but names should not be the only place where
scientific context lives. Snapshots, refs, proposals, or future run bindings may
carry structured context such as:

```text
device
sample
cooldown
campaign
temperature_regime
experiment_type
simulation_model
```

Context metadata should identify or label parameter state; it should not make
the parameter registry own sample records, device state, workflow definitions,
or simulation artifacts.

## API Use Cases

The parameter API should support direct use and system integration without
forcing both audiences through the same amount of ceremony.

### Direct Python Registry Use

This path is for users who manage parameters directly from scripts or notebooks.
It should optimize for the public mental model: profile, snapshot, tree/table
sections, draft, diff, and commit.

```python
params = ws.parameters

# Resolve a mutable profile/ref to an immutable snapshot.
snapshot = params.get_snapshot(ref="main")

# Read tree and table values.
ifbw = snapshot.get("/devices/vna/init/if_bandwidth")
gate = snapshot.table("one_qubit_gates").get(qubit="q3", gate="x90")

# Edit through a draft.
draft = params.create_draft(base="main")
draft.set("/devices/vna/init/if_bandwidth", 3000.0)
draft.table("one_qubit_gates").update(
    key={"qubit": "q3", "gate": "x90"},
    values={"amplitude": 0.127},
)
new_snapshot = draft.commit("Tune q3 x90 and VNA IF bandwidth")

# Move a ref with optimistic locking.
params.update_ref(
    "main",
    new_snapshot.id,
    expected_old_snapshot=snapshot.id,
)
```

Direct use should also cover import, export, history, and diff:

```python
diff = params.diff("p_old", "p_new", scope="/devices/vna")
patch = diff.select(paths=["/devices/vna/init/if_bandwidth"])

draft = params.create_draft(base="main")
draft.apply(patch)
draft.validate().raise_for_errors()
snapshot = draft.commit("Update VNA initialization defaults")
```

### Run System Integration

This path is for experiment or workflow systems that need parameter state as an
input. The parameter registry should provide resolved bindings and validation,
but the run system should own run-local inputs, effective configuration, command
planning, dataset creation, and device interaction.

```python
binding = params.resolve("main")

snapshot = params.read_snapshot(binding.snapshot_id)
validation = params.validate_overrides(
    snapshot_id=binding.snapshot_id,
    overrides=run_overrides,
)
validation.raise_for_errors()

run.record_parameter_binding(binding)
```

The run system then owns this step:

```text
parameter snapshot
  + run-local parameters
  + runtime overrides
  -> effective run config
  -> compiled execution plan
  -> datasets and run provenance
```

The registry should not compile waveforms, plan scans, apply instrument
commands, or decide run state transitions.

### Analysis And Calibration Integration

Analysis, calibration, optimizers, and future AI assistants should propose
parameter changes as patches. The registry should store the proposal, preserve
its source links, apply it to a draft, validate it, and commit it only through
an explicit mutation path.

Analysis should be represented as a producer/consumer activity when that model
exists: it consumes datasets, runs, parameter snapshots, or artifacts and
produces analysis results, processed datasets, reports, or parameter proposals.
Calibration workflows should depend on those analysis outputs rather than
embedding analysis results inside the original experiment run.

```python
proposal = params.create_proposal(
    base_snapshot=binding.snapshot_id,
    patch=patch,
    source={
        "kind": "analysis",
        "run_id": run.id,
        "dataset_id": dataset.id,
        "analysis_id": analysis.id,
    },
)

draft = params.apply_proposal_to_draft(proposal.id)
draft.validate().raise_for_errors()
new_snapshot = draft.commit("Apply rabi calibration result")
```

Promotion to a mutable profile/ref should remain a separate action:

```python
params.update_ref(
    "calibration/rabi/latest",
    new_snapshot.id,
    expected_old_snapshot=binding.snapshot_id,
)
```

### Minimal Experiment Convenience

Higher-level experiment helpers may offer a compact API, but it should still
resolve refs before execution and store the resulting parameter binding:

```python
with experiment.run(parameter_ref="main") as run:
    # Fricon resolves main -> immutable snapshot before execution.
    # The run record stores both the source ref and resolved snapshot.
    ...
```

This is convenience over the run integration boundary, not an invitation for
the parameter registry to own experiment execution.

Typed Python helper generation is a useful later enhancement, not an MVP
blocker. Helpers should bind to a schema snapshot and include runtime
compatibility checks when following mutable refs.

## Storage Direction

This proposal does not choose a storage implementation.

A first implementation should optimize for correctness, history, diff,
validation, and human review rather than high write throughput. SQLite-backed
metadata and value tables may be sufficient for early parameter snapshots.

If table sections become very large, table snapshot payloads can later move to
Arrow IPC or Parquet artifacts while relational metadata continues to own
snapshot identity, refs, schema, keys, and history.

Any implementation that changes workspace layout, SQLite schema, IPC contracts,
Python API contracts, or desktop DTOs must follow the corresponding maintenance
checklists.

## Feature Shaping

### Feature 0: ADR And Product Boundary

Classification: `after dataset semantics`.

Scope:

- decide durable terminology
- decide snapshot/ref/draft/patch invariants
- decide storage and migration strategy
- decide direct Python API and integration API boundaries
- decide how run records freeze resolved snapshots

### Feature 1: Run-Linked Parameter Snapshot References

Classification: `after dataset semantics`, after minimal run records exist.

Scope:

- expose parameter bindings that resolve source refs to immutable snapshot IDs
  and hashes
- record source parameter ref and resolved immutable snapshot ID in run records
- record run-local parameters
- record runtime overrides separately from run-local parameters
- store effective config hash or serialized effective config

Out of scope:

- full parameter registry UI
- analysis-driven update proposals
- direct instrument apply

### Feature 2: Minimal Parameter Registry

Classification: `later`.

Scope:

- immutable snapshots
- refs and tags
- drafts
- tree CRUD
- table CRUD with required keys
- basic schema metadata
- rough tree/table import with inferred schema
- canonical unit metadata and display-unit separation
- stable `param_id`, `table_id`, and `column_id`
- logical deletion and tombstones
- snapshot-to-snapshot diff
- selected diff apply to draft
- minimal draft conflict detection

Out of scope:

- full Git-like merge
- typed helper generation
- first-class ndarray parameters
- direct hardware apply

### Feature 3: Analysis Proposals And Calibration Flow

Classification: `later`.

Scope:

- analysis result to parameter proposal links
- proposal review and validation
- commit provenance from run, dataset, and analysis IDs
- ref promotion policies

Out of scope:

- broad workflow scheduler
- device driver framework
- multi-user approval system

### Feature 4: Typed Helpers And Impact Analysis

Classification: `future concept`.

Scope:

- generated path constants and table row dataclasses
- schema snapshot compatibility checks
- used-parameter manifest
- stale parameter and impact analysis views

## Open Questions

- Should the public term be `profile`, `ref`, or both with one treated as the
  Python/API spelling?
- Should a workspace have one parameter registry or multiple named registries?
- How much schema metadata belongs in the MVP: dtype only, or dtype, unit,
  nullable, constraints, description, and lifecycle status?
- What should be available in the direct Python API versus the stricter
  integration API?
- Should table row keys be immutable within a snapshot lineage, or can key
  changes be represented as row moves?
- Should every run persist a full effective config, only a hash, or both?
- When should runtime overrides be allowed, warned, or blocked?
- Which parameter changes require human review before moving `main`?
- How should parameter refs compose with future sample, device, cooldown, or
  workflow definitions?
- Which structured context fields should be first-class versus free-form
  metadata?
- What belongs in parameter snapshots versus code, environment, device state,
  datasets, or run-local inputs?

## Design Principles

1. The registry stores long-lived shared parameters, not every one-off run
   input.
2. Runs freeze immutable parameter snapshots, not only mutable refs.
3. A snapshot is one coherent parameter state with an optional tree section and
   multiple table sections.
4. Schema changes are first-class operations.
5. Path and name are editable addresses; stable IDs are identity.
6. Diffs are structured, refactor-aware, and apply to drafts.
7. Runtime overrides are recorded as run facts and are not silent registry
   mutations.
8. Parameter updates should flow through proposals, validation, and commits.
9. Direct hardware application is separate from parameter commit creation.
10. Large arrays and simulation artifacts should initially be represented by
    dataset or artifact references, not embedded parameter values.
11. The registry exposes bindings and validation for other systems, but
    run/execution systems own effective configs and device interaction.
12. Unit-aware values need canonical storage semantics; display unit changes
    should not masquerade as scientific value changes.
13. Users can begin with rough imports and inferred schema, then refine
    structure through normal draft and commit workflows.
