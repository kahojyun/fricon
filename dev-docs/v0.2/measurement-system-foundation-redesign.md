# Measurement System Foundation Redesign

## Status

Supporting pre-adoption redesign proposal for the v0.2 reset.

This is not current behavior. Do not implement or document behavior from this
note as user-facing functionality until the relevant ADR, storage, IPC, Python
API, desktop UI, migration, and release decisions have landed.

Read `README.md` and `design.md` first. This note preserves the broader
reasoning behind the reset; `design.md` is the cleaner canonical synthesis for
new planning.

Canonical v0.2 naming now prefers `Measurement` for the public data-taking
record. Historical `Experiment` or `ExperimentRun` wording in this supporting
note should be read as older proposal terminology unless a later ADR chooses a
different public noun.

This note intentionally allows large breaking changes while Fricon has not yet
entered real lab use. It may revise or supersede parts of
`../dataset-semantic-architecture-proposal.md` and
`../adr/0002-decide-dataset-semantic-manifest-v1.md` if the broader
measurement-system model requires a different foundation.

## Purpose

Fricon should become a practical local scientific measurement record system,
not only a table writer with charting. The first adoption milestone is replacing
a simple LabRAD Grapher/Data Vault style logger for new measurement work while
leaving room for later analysis, simulation, import, calibration, workflow, and
AI-assisted automation.

The redesign should establish the durable model before implementation hardens:

- what a dataset is
- what a run-like producer or consumer record is
- what users see in V1
- what remains internal provenance or debug detail
- which breaking dataset changes are acceptable now
- how future automatic calibration can build on measured data, analysis
  results, and parameter proposals without retrofitting the storage model

## Primary User Mental Model

V1 should optimize for this simple mental model:

```text
I ran a measurement.
It produced datasets.
Fricon helps me inspect, annotate, recover, and reopen them.
```

The recommended measurement path should be measurement-first once the run API
exists. Sample/session context should be selectable as an active default, not a
hard prerequisite for quick measurements:

```python
lib = fricon.library()
lib.use_context(sample="sample-a", session="cooldown-2026-05")

with lib.measurement("cooldown sweep") as meas:
    s21 = meas.dataset("s21")
    noise = meas.dataset("noise")

    for freq in freqs:
        s21.write(freq=freq, signal=measure_s21(freq))
        noise.write(freq=freq, noise=measure_noise(freq))
```

The measurement context should own default finalization for datasets opened
through it. On normal measurement exit, open produced datasets are finished. On
exceptional measurement exit, open produced datasets are aborted or marked
suspect according to the settled lifecycle policy. Individual datasets may still
be explicitly finished or aborted earlier when a multi-output measurement needs
per-output control.

Nested dataset context managers may remain available for advanced explicit
lifecycle control, but public V1 examples should prefer the flatter
measurement-scoped writer form when it is sufficient.

Quick measurements without sample context and the lower-level dataset-only path
should remain valid:

```python
lib = fricon.library()

with lib.measurement("quick check") as meas:
    ds = meas.dataset("quick_table")
    ds.write(x=1.0, y=2.0)

with lib.dataset("scratch_table") as ds:
    ds.write(x=1.0, y=2.0)
```

Dataset-only creation should create an unassigned dataset unless the caller
explicitly links it to a run-like record. The dataset storage layer should not
silently create measurement records.

## User-Visible Concepts

Keep the V1 user model small:

- `Data Library`
- optional active `Sample` and `Sample Session`
- `Measurement`
- `Dataset`
- notes, tags, pin or favorite state, and quality state

Later user-visible concepts may include:

- `Analysis`
- `Parameter profile` and `Parameter snapshot`
- `Calibration workflow`

Keep these mostly internal, advanced, or debug-facing:

- `ScriptRun`
- `DatasetWriteSession`
- `RunInput`
- `RunOutput`
- `TaskQueueEntry`
- `ResourceLease`
- storage chunks, manifests, sidecars, IPC protocol versions, and SQLite tables

The implementation may use a shared internal shape for run-like records, but
public docs should not lead with a generic `RunRecord` abstraction. Use concrete
user terms when they matter: measurement, analysis, import, simulation, and
calibration.

## Notes, Tags, And Quality

Users should not need to tag or annotate every object.

Default rule:

```text
Put notes, tags, and quality on the highest meaningful work record.
```

For V1 this usually means the measurement record:

- measurement-level labels, notes, tags, pin or favorite state, and quality live
  on `Measurement`
- datasets keep dataset-local semantics and output-specific exceptions
- dataset notes or quality are useful when one output differs from the whole
  run

Examples:

- "cooldown sweep #12" belongs on the measurement
- "publication candidate" usually belongs on the measurement
- "the S21 output is good but the noise trace is bad" belongs on the affected
  dataset
- future "fit used a bad initial guess" belongs on the analysis result or
  analysis run

## Core Model

Use this conceptual model:

```text
DataLibrary
  -> concrete run records
       kind: measurement | analysis | import | simulation | calibration
       inputs
       outputs
       notes/tags/quality
       metadata
       parameter binding when relevant
  -> dataset artifacts
       kind: measured | processed | imported | simulation
       dataset facts
       dataset semantics
       projections
       provenance links
```

Use `WorkspaceRoot` only as internal or legacy implementation terminology.

The central rule is:

```text
Dataset is not the measurement record.
Dataset is a durable data artifact produced or consumed by run-like activities.
```

A measurement may produce datasets. An analysis may consume measured datasets
and produce processed datasets, figures, reports, metrics, or parameter
proposals. A calibration workflow may coordinate measurement, analysis,
validation, and promotion. None of these require datasets to be owned
exclusively by measurements.

## Dataset Artifact

A dataset artifact should have stable identity and independent lifecycle:

- data-library-local integer ID
- stable UID
- name and optional description
- `kind`: at least reserve `measured`, `processed`, `imported`, and
  `simulation`
- lifecycle/status
- quality summary or output-local quality state
- current semantic manifest or manifest reference
- producer and consumer provenance links
- archive/export compatibility metadata

The dataset remains directly openable from Python and the desktop UI whether it
is produced by a measurement, analysis, import, simulation, calibration
workflow, or lower-level dataset-only code.

## Dataset Facts, Semantics, And Projections

Keep three layers separate:

```text
dataset facts       -> append-only recorded rows and sidecars
dataset semantics   -> column meaning, scan meaning, units, labels, display hints
dataset projections -> table, chart, grid, heatmap, live-view, and duplicate handling
```

### Dataset Facts

Rows are append-only facts. Do not overwrite rows to represent retry, resume,
correction, or invalidation.

Breaking changes worth making now:

- materialize a stable `__ds_record_id` for new datasets
- reserve a Fricon-owned system-field prefix such as `__ds_`
- make append order durable across restart, export, and import
- support optional logical-index sidecars for scans where append order is not
  enough
- keep raw storage independent of chart heuristics

### Dataset Semantics

Dataset semantics should describe dataset-local meaning:

- column names and dtypes
- units, labels, hidden-by-default state, and chart-axis hints
- scan axes and logical index realization
- duplicate-position policy
- default view hints when appropriate

Dataset semantics should not own measurement context, analysis context,
parameter history, device driver state, or workflow state.

### Dataset Projections

Charts, heatmaps, live views, and grid readers should consume resolved
interpretation, not infer meaning from row adjacency or field order.

Duplicate logical positions should be resolved in one projection layer. The
default v1 policy can remain `latest_by_record_id`.

## Run-Like Records And Provenance

Concrete run records should consume inputs and produce outputs:

```text
MeasurementRun
  outputs: measured Dataset

AnalysisRun
  inputs: measured Dataset | MeasurementRun | ParameterSnapshot | Artifact
  outputs: processed Dataset | AnalysisResult | Report | ParameterProposal

ImportRun
  inputs: external source summary
  outputs: imported Dataset

SimulationRun
  inputs: parameter snapshot | simulation config | code reference
  outputs: simulation Dataset | AnalysisResult

CalibrationRun or WorkflowRun
  inputs: parameter profile/ref, workflow definition
  outputs: measurement runs, analysis runs, parameter proposals, promoted refs
```

Implementation can share provenance edge tables such as `RunInput` and
`RunOutput`, but those should not become primary V1 user concepts.

`Artifact` should be the general provenance concept. `Dataset` is the primary
artifact subtype needed for the first LabRAD-style replacement slice. Future
logs, reports, figures, attachments, code summaries, waveform/configuration
files, and device snapshots should not be forced into dataset metadata.

## Automatic Calibration Shape

Clean automatic calibration should be modeled as workflow over provenance:

```text
CalibrationWorkflowDefinition
  -> CalibrationWorkflowRun
      -> ManagedMeasurementRun produces measured Dataset
      -> AnalysisRun consumes measured Dataset
      -> AnalysisRun produces AnalysisResult and ParameterProposal
      -> ValidationResult checks proposal
      -> Approval or policy gate promotes snapshot to ParameterRef
```

Calibration should not silently mutate `main` or any recommended parameter
profile during data collection. Even when a future policy allows auto-commit to
a calibration profile, the mutation path should remain explicit:

```text
analysis result -> parameter proposal -> validation -> draft/commit -> ref update
```

## Breaking Changes To Consider Now

Because Fricon is pre-adoption, breaking changes are acceptable when they remove
long-term ambiguity:

- redesign dataset catalog schema around dataset artifacts and provenance links
- make dataset manifests mandatory for new datasets
- materialize record IDs and reserve system fields
- replace inference-led chart semantics with resolved interpretation
- add dataset `kind` and producer/consumer provenance affordances early
- revise Python creation APIs to distinguish measurement-produced datasets from
  unassigned datasets
- change archive layout to include dataset semantics and provenance summaries
- change desktop DTOs to expose resolved semantics and run links instead of
  coarse chart flags
- bump workspace, archive, and IPC versions as needed

Do not use breaking-change freedom to make simple measurement scripts heavy.
The low-friction write path must survive.

## V1 Scope

V1 should expose:

- local data library
- data library UUID and user-editable display name for source provenance
- optional active sample/session context with attach-later correction
- interactive measurement records
- measured dataset artifacts as the first concrete artifact subtype
- dataset table and chart inspection
- explicit dataset semantics enough for robust scan/chart behavior
- measurement-scoped dataset writer handles with default finalization tied to
  the measurement context
- run-level notes, tags, pin/favorite state, quality state, and legacy JSON
  metadata
- dataset-local notes or quality only for output-specific exceptions
- generated Python snippets or stable IDs for reopening data
- measurement-centered export bundles that can be opened directly from Python
  or a read-only GUI viewer without import into another data library
- interrupted-data recovery with explicit continuation, validation, or
  invalidation

V1 should not require:

- full legacy LabRAD/Data Vault import
- generic workflow DAG execution
- managed runner, task queue, or resource leases
- full parameter registry
- device driver framework
- automatic Git, `uv`, or `pixi` environment management
- analysis-run UI
- automatic calibration

## Implementation Staging

Suggested staged path:

1. Foundation ADR
   - Decide whether this note supersedes or revises ADR 0002.
   - Commit to dataset artifact identity, dataset fact invariants, semantic
     manifest boundaries, and provenance edge direction.

2. Dataset artifact and semantics foundation
   - Implement new dataset identity/lifecycle shape.
   - Implement durable record IDs, manifests, and resolved interpretation.
   - Preserve low-friction Python writes.

3. Interactive measurement records
   - Add explicit measurement context.
   - Link produced datasets through provenance, not ownership.
   - Keep dataset-only creation as unassigned.

4. Desktop V1 replacement flow
   - Run-first measurement browsing.
   - Dataset table/chart/detail views.
   - Run notes/tags/quality and output-specific dataset exceptions.
   - Python read snippets.

5. Later derived-data foundation
   - Add analysis/import/simulation run taxonomy only when needed.
   - Add shared input/output provenance edges.
   - Add parameter proposals and calibration workflow shape after the parameter
     foundation exists.

## Open Questions

- Should internal storage use one shared `run_record` table with `kind`, or
  separate tables for experiment, analysis, import, simulation, and calibration
  records with shared provenance edge tables?
- Which dataset `kind` values are required in the first schema?
- Should unassigned datasets be first-class in the desktop UI or mostly visible
  through dataset search and debugging views?
- How much provenance should dataset archives include in V1?
- What should the first portable measurement export bundle format include beyond
  source data library identity, measurement metadata, dataset artifacts, selected
  non-table artifacts, provenance summaries, and checksums?
- Should output-specific notes and quality state live directly on datasets, or
  on run-output link records?
- How should dataset continuation interact with completed measurement runs?
- What compatibility promise, if any, should exist for pre-redesign local test
  workspaces?
- Which decisions require a new ADR versus revising ADR 0002?
