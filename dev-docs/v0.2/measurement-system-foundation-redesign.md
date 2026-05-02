# Measurement System Foundation Redesign

## Status

Proposed pre-adoption redesign direction.

This is not current behavior. Do not implement or document behavior from this
note as user-facing functionality until the relevant ADR, storage, IPC, Python
API, desktop UI, migration, and release decisions have landed.

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
I ran an experiment.
It produced datasets.
Fricon helps me inspect, annotate, recover, and reopen them.
```

The recommended measurement path should be experiment-first once the run API
exists:

```python
with ws.experiment("cooldown sweep") as run:
    s21 = run.dataset("s21")
    noise = run.dataset("noise")

    for freq in freqs:
        s21.write(freq=freq, signal=measure_s21(freq))
        noise.write(freq=freq, noise=measure_noise(freq))
```

The experiment context should own default finalization for datasets opened
through it. On normal experiment exit, open produced datasets are finished. On
exceptional experiment exit, open produced datasets are aborted or marked
suspect according to the settled lifecycle policy. Individual datasets may still
be explicitly finished or aborted earlier when a multi-output experiment needs
per-output control.

Nested dataset context managers may remain available for advanced explicit
lifecycle control, but public V1 examples should prefer the flatter
experiment-scoped writer form when it is sufficient.

The lower-level dataset-only path should remain valid:

```python
with ws.dataset("quick_table") as ds:
    ds.write(x=1.0, y=2.0)
```

Dataset-only creation should create an unassigned dataset unless the caller
explicitly links it to a run-like record. The dataset storage layer should not
silently create experiment records.

## User-Visible Concepts

Keep the V1 user model small:

- `Workspace`
- `Experiment`
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
user terms when they matter: experiment, analysis, import, simulation, and
calibration.

## Notes, Tags, And Quality

Users should not need to tag or annotate every object.

Default rule:

```text
Put notes, tags, and quality on the highest meaningful work record.
```

For V1 this usually means the experiment run:

- experiment-level labels, notes, tags, pin or favorite state, and quality live
  on `Experiment`
- datasets keep dataset-local semantics and output-specific exceptions
- dataset notes or quality are useful when one output differs from the whole
  run

Examples:

- "cooldown sweep #12" belongs on the experiment
- "publication candidate" usually belongs on the experiment
- "the S21 output is good but the noise trace is bad" belongs on the affected
  dataset
- future "fit used a bad initial guess" belongs on the analysis result or
  analysis run

## Core Model

Use this conceptual model:

```text
Workspace
  -> concrete run records
       kind: experiment | analysis | import | simulation | calibration
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

The central rule is:

```text
Dataset is not the experiment record.
Dataset is a durable data artifact produced or consumed by run-like activities.
```

An experiment may produce datasets. An analysis may consume measured datasets
and produce processed datasets, figures, reports, metrics, or parameter
proposals. A calibration workflow may coordinate measurement, analysis,
validation, and promotion. None of these require datasets to be owned
exclusively by experiments.

## Dataset Artifact

A dataset artifact should have stable identity and independent lifecycle:

- workspace-local integer ID
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
is produced by an experiment, analysis, import, simulation, calibration
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

Dataset semantics should not own experiment context, analysis context,
parameter history, device driver state, or workflow state.

### Dataset Projections

Charts, heatmaps, live views, and grid readers should consume resolved
interpretation, not infer meaning from row adjacency or field order.

Duplicate logical positions should be resolved in one projection layer. The
default v1 policy can remain `latest_by_record_id`.

## Run-Like Records And Provenance

Concrete run records should consume inputs and produce outputs:

```text
ExperimentRun
  outputs: measured Dataset

AnalysisRun
  inputs: measured Dataset | ExperimentRun | ParameterSnapshot | Artifact
  outputs: processed Dataset | AnalysisResult | Report | ParameterProposal

ImportRun
  inputs: external source summary
  outputs: imported Dataset

SimulationRun
  inputs: parameter snapshot | simulation config | code reference
  outputs: simulation Dataset | AnalysisResult

CalibrationRun or WorkflowRun
  inputs: parameter profile/ref, workflow definition
  outputs: experiment runs, analysis runs, parameter proposals, promoted refs
```

Implementation can share provenance edge tables such as `RunInput` and
`RunOutput`, but those should not become primary V1 user concepts.

## Automatic Calibration Shape

Clean automatic calibration should be modeled as workflow over provenance:

```text
CalibrationWorkflowDefinition
  -> CalibrationWorkflowRun
      -> ManagedExperimentRun produces measured Dataset
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
- revise Python creation APIs to distinguish experiment-owned datasets from
  unassigned datasets
- change archive layout to include dataset semantics and provenance summaries
- change desktop DTOs to expose resolved semantics and run links instead of
  coarse chart flags
- bump workspace, archive, and IPC versions as needed

Do not use breaking-change freedom to make simple measurement scripts heavy.
The low-friction write path must survive.

## V1 Scope

V1 should expose:

- local workspace
- interactive experiment records
- measured dataset artifacts
- dataset table and chart inspection
- explicit dataset semantics enough for robust scan/chart behavior
- experiment-scoped dataset writer handles with default finalization tied to
  the experiment context
- run-level notes, tags, pin/favorite state, quality state, and legacy JSON
  metadata
- dataset-local notes or quality only for output-specific exceptions
- generated Python snippets or stable IDs for reopening data
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

3. Interactive experiment records
   - Add explicit experiment context.
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
- Should output-specific notes and quality state live directly on datasets, or
  on run-output link records?
- How should dataset continuation interact with completed experiment runs?
- What compatibility promise, if any, should exist for pre-redesign local test
  workspaces?
- Which decisions require a new ADR versus revising ADR 0002?
