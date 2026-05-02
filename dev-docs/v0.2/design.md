# Fricon v0.2 Design

## Status

Canonical v0.2 design synthesis.

This is proposed direction, not current behavior. Use this document as the
first v0.2 reference before reading the longer supporting proposals in this
directory.

## Product Thesis

Fricon v0.2 should be a local lab data library and automation foundation for
scientific measurement work.

The goal is not to build a heavier database browser. The goal is to help
experimentalists move quickly while keeping enough structure that samples,
cooldowns, code, parameters, calibration, and data do not quietly diverge.

The first adoption target remains practical:

```text
replace a simple LabRAD Grapher/Data Vault style logger for new experiments
```

The long-term target is broader:

```text
fast exploration
  -> trustworthy measurement history
  -> large parameter-set management
  -> reviewable calibration automation
  -> eventual device control without LabRAD
```

## What v0.2 Optimizes For

v0.2 should optimize for these user outcomes:

- run exploratory experiments from Python with little boilerplate
- keep one coherent local data library instead of many data/code folders
- record sample and cooldown/session context beside measurements
- let users set active sample/session context from a notebook prelude or UI
  without making sample setup a hard requirement for quick experiments
- inspect live and historical datasets through desktop UI and Python
- preserve parameter, code, environment, and quality context for each run
- visualize sample/device parameters on a 2D map when the lab model needs it
- automate repeated calibration without silent parameter mutation
- leave a clean path to managed device communication after LabRAD is removed

v0.2 should not optimize for:

- hosted SaaS
- lab-wide administration
- full multi-user permission management
- a generic workflow DAG engine
- a broad hardware driver framework in the first implementation

## User-Visible Concept Budget

Fricon should not ask users to understand every internal provenance object.

User-visible concepts for the first v0.2 slice:

- Data Library: the local Fricon root and catalog
- Sample: the physical object, device, chip, wafer, batch, or specimen; useful
  when known, but not required before every quick experiment
- Sample Session: a cooldown, mounting, wiring, probing, or campaign context;
  can be selected as active context or attached after a run
- Experiment: the normal record for measurement work
- Dataset: a table-shaped artifact produced or consumed by work

User-visible concepts to preserve for later v0.2 work:

- Parameter Profile: a named mutable reference to a useful parameter state
- Parameter Snapshot: immutable parameter facts captured for a run
- Analysis: later work that consumes datasets and may produce results,
  datasets, reports, or parameter proposals
- Calibration: a reviewable workflow that turns measurements and analysis into
  parameter proposals or approved profile updates

Concepts that should mostly remain internal or advanced:

- dataset write session
- script run
- scan point identity
- resource lease
- device apply plan
- readback record
- task queue entry
- actor token
- storage manifest
- projection or resolved chart interpretation

This boundary matters. Users should not have to tag, note, or name every
dataset write session, script run, scan point, or internal device record.

## Notes, Tags, And Quality

Notes, tags, favorites, and quality flags should attach to the level where
users naturally make decisions.

Default user-facing annotation levels:

- sample: identity, preparation, layout, long-lived notes, aliases, tags
- sample session: cooldown/setup context, wiring, drift notes, session quality
- experiment: hypothesis, run notes, tags, quality, interruption decisions
- analysis or calibration: conclusion, accepted/rejected proposals, audit note
- dataset: output-local exceptions, display hints, or dataset-specific issues

Do not require users to annotate every dataset produced by an experiment.
Dataset-level notes are useful when an output has a local issue or meaning, but
the experiment is the default annotation container for measurement work.

## Domain Model

The clean v0.2 domain model is:

```text
DataLibrary
  Sample
    SampleSession
      Experiment
        Artifact
          DatasetArtifact
  Analysis
    Artifact
      DatasetArtifact
      AnalysisResult
      ParameterProposal
  CalibrationWorkflow
    Experiment
    Analysis
    ParameterProposal
    ParameterProfile update
```

More formally:

```text
ActivityRun consumes inputs and produces artifacts.

Experiment, Analysis, Import, Simulation, and Calibration are user-facing or
workflow-facing activity types.

Artifact is the general output or input concept. DatasetArtifact is the primary
table-shaped artifact for v0.2, while AnalysisResult, Report,
ParameterProposal, file attachments, logs, figures, code summaries, and future
device snapshots should fit the same provenance pattern.
```

The `ActivityRun` pattern should be an internal modeling tool, not the primary
word users see in the UI. Users can still see Experiment, Analysis, Import,
Simulation, and Calibration as concrete work types.

## Experiment And Simulation

Do not force experiment and simulation into completely separate foundations.

The clean distinction is:

- Experiment: activity that usually measures the physical world or hardware
- Simulation: activity that computes synthetic or model-derived data
- ActivityRun: shared internal provenance pattern for both
- DatasetArtifact: table-shaped output artifact that can be measured,
  processed, imported, or simulated

This gives users familiar labels while keeping storage, lineage, and analysis
code reusable.

## Artifact And Dataset Model

An artifact is a durable output or input linked through provenance.

The first v0.2 implementation should focus on `DatasetArtifact`, because
LabRAD-style replacement needs table-shaped measured data, live plots, and
Python reopening. The model should still reserve room for non-table artifacts
such as:

- reports
- figures
- logs
- attachments
- code and environment summaries
- waveform or configuration files
- future device snapshots and readback summaries

A dataset is an artifact, not the whole experiment record.

Dataset artifacts should have stable identity, table facts, dataset-local
semantics, and direct Python access. They should not own sample identity,
experiment intent, parameter history, code state, or calibration decisions.

Keep the dataset stack split into:

```text
dataset facts
  append-only Arrow-compatible records

dataset semantics
  column meaning, units, scan axes, logical indices, display hints

dataset projections
  resolved grid/chart/live-view interpretation
```

The active dataset semantic proposal remains relevant for record IDs,
append-only facts, manifests, and resolved interpretation. Before committing
more durable dataset APIs or storage contracts, reconcile that work with this
broader v0.2 data-library and provenance model.

## Experiment-Scoped Writes

The common write path should share the experiment lifecycle.

Prefer this shape for normal measurements:

```python
with fricon.library() as lib:
    # Optional prelude, similar to the old habit of setting data_dir at the
    # top of a notebook. The exact API is not settled.
    lib.use_context(sample="sample-a", session="cooldown-2026-05")

    with lib.experiment("rabi q3") as exp:
        rabi = exp.dataset("rabi")
        chevron = exp.dataset("chevron")

        for amp in amps:
            rabi.write(amp=amp, response=measure_rabi(amp))

        for freq, amp in points:
            chevron.write(freq=freq, amp=amp, signal=measure(freq, amp))
```

The active context may be set from a notebook prelude, CLI, desktop UI, or an
explicit Python object. It should behave like a helpful default, not hidden
provenance. Each experiment should record the resolved sample/session IDs or
record that no sample context was selected.

Quick experiments should also be valid without sample context:

```python
with fricon.library() as lib:
    with lib.experiment("quick resonator check") as exp:
        s21 = exp.dataset("s21")
        s21.write(freq=7.1e9, signal=measure())
```

Users should be able to attach or correct sample/session links later through an
auditable correction path.

Dataset handles opened from an experiment should finalize with the experiment
unless the user explicitly aborts or detaches them. This avoids nested writer
ceremony for the normal case while preserving explicit dataset lifecycle for
lower-level or streaming APIs.

Lower-level dataset creation remains useful:

```python
with lib.dataset("scratch") as ds:
    ds.write(x=1.0, y=2.0)
```

Lower-level datasets are unassigned artifacts until linked to a producer.

## Sample And Session Model

The sample model should replace the old habit of using data-vault directories
as sample boundaries.

Sample/session context should be easy to set but not mandatory for quick
measurements. Many existing notebooks start with a small prelude such as
`data_dir = ...`; Fricon should support an equivalent active sample/session
prelude and a desktop active-context selector.

An experiment may start with:

- an explicit sample/session context
- the current active sample/session context
- no sample/session context, with attach-later support

Use one `Sample` when the physical object is the same object. Use multiple
`SampleSession` records for cooldowns, mountings, wiring configurations,
probing sessions, or other periods where conditions may drift.

Create a new sample when:

- the physical object changed
- processing created a meaningfully new object
- the lab would not want old and new measurements compared as the same object

Do not create a new sample only because the cooldown changed. Put cooldown
drift, setup changes, and session-local notes on the sample session and
parameter history.

Sample records should allow lightweight custom fields and optional 2D layout
visualization. This should cover the current need for small custom GUIs that
edit JSON sample parameters and color a 2D graph by parameter or result.

## Parameter Model

Large parameter sets should be explicit and versioned, but users should not
need to design a full parameter schema before collecting exploratory data.

The clean lifecycle is:

```text
ParameterProfile
  mutable named ref used by humans

ParameterSnapshot
  immutable resolved facts captured for a run

ParameterProposal
  analysis/calibration output describing a candidate change

Profile update
  explicit promotion after validation or approval
```

Rules:

- experiment facts should link to immutable snapshots, not mutable JSON files
- run-local overrides should be recorded on the run
- analysis and calibration should produce proposals before profile updates
- automatic calibration should not silently mutate important profiles during
  measurement
- parameter display, diff, and search should be first-class UI concerns after
  the minimal experiment and dataset foundation exists

## Code And Environment Provenance

v0.2 should reduce code-directory copying without becoming a full Git or
environment manager.

Start with passive summaries:

- script path or module entry point
- Git commit, dirty state, or file hash summary when available
- Python version
- Fricon version
- lock file or environment summary when practical

The record should help a researcher answer what probably ran, while making
clear when exact reproducibility is not guaranteed.

## Managed Experiment Model

Simple experiments should remain ordinary Python.

Managed experiments should be an optional integration path for repeated,
retryable, automation-heavy, or calibration-critical work. The model should
make the intended hardware state inspectable before execution:

```text
parameter snapshot
  + run-local inputs
  + scan point
  -> desired device state
  -> device apply plan
  -> readback
  -> measurement
  -> dataset writes
  -> post-processing hooks
```

This is similar to a declarative UI model: user code describes the desired
state for a scan point, and Fricon reconciles that desired state with devices,
datasets, and provenance.

The benefit is not abstraction for its own sake. The benefit is:

- preview the state that will be sent to devices
- dry-run against dummy devices
- diff desired state against current or last-applied state
- resume after interruption with explicit scan point identity
- attach point/sweep/dataset post-processing hooks
- build automatic calibration on auditable measurements and proposals

Imperative experiment code remains valid. It just gets less automatic help with
retry, resume, dry-run, and device-state inspection unless it opts into the
managed API or explicit advanced hooks.

## Device Evolution Path

v0.2 should be able to evolve toward Fricon-managed device communication and
eventually remove LabRAD from the measurement stack.

Do not build the full driver framework first. Reserve a minimal boundary now:

- DeviceIdentity: stable logical device name and backend binding
- DeviceCapability: declared operations, readable state, writable state, and
  limits exposed by a device adapter
- DeviceAdapter: local boundary that can later wrap LabRAD, direct Python
  drivers, VISA, serial, vendor SDKs, or dummy devices
- DeviceState: current or observed state
- DesiredDeviceState: state requested by a managed scan point
- ApplyPlan: ordered changes Fricon intends to send
- Readback: what hardware reported after apply
- ResourceLease: minimal guard for devices that cannot be used concurrently

The first implementation can record only summaries, placeholders, or a very
thin adapter contract. The important decisions are:

- device state belongs to managed execution provenance, not dataset metadata
- Fricon should have a typed place for future device capabilities and readbacks
- dummy devices and dry runs should use the same boundary as real adapters
- LabRAD compatibility, if needed during migration, should be an adapter behind
  the boundary rather than the conceptual model

## Analysis And Calibration

Analysis is a consumer and producer.

It may consume experiment artifacts, usually datasets, and produce:

- derived datasets
- scalar or structured results
- reports
- quality decisions
- parameter proposals

Calibration is a workflow over experiment and analysis records:

```text
CalibrationWorkflowRun
  -> experiment or managed experiment
  -> measured dataset
  -> analysis result
  -> parameter proposal
  -> validation
  -> approval or policy gate
  -> parameter profile update
```

This keeps experiment outputs clean while still letting the UI show calibration
history inside the broader experiment/sample context.

## Distribution And Runtime

v0.2 should expose three public surfaces:

- desktop GUI
- `fricon` CLI
- Python SDK

They should operate against one local data library and one local service:

```text
desktop GUI / CLI / Python SDK
  -> local Fricon service
  -> Fricon data library
```

The desktop GUI may be implemented as a web application packaged with Tauri.
Feature code should avoid unnecessary Tauri-only assumptions so the same UI can
support remote monitoring or browser access later.

## Remote And Auth Boundary

Remote access should be planned early, but v0.2 should not become a multi-user
system.

The recommended boundary is:

- local loopback access can use implicit local trust or a generated local token
- remote access requires token or pairing
- mutating operations record an actor
- the first authorization model is single-owner
- roles, teams, and permission matrices remain future scope

Auth is hard to retrofit when APIs and audit logs have no actor field. Add the
actor boundary early; defer full multi-user product complexity.

## Rewrite Strategy

Do not create a permanent `fricon-v2` project.

Use the same repository and make a deliberate domain-model reset. Short-lived
prototype crates or scratch branches are acceptable for schema/API spikes, but
durable code should return to this repo so tests, release process, packaging,
and migration decisions stay aligned.

High rewrite-risk areas:

- public workspace mental model
- dataset catalog schema
- dataset metadata ownership
- Python dataset creation API
- desktop dataset-first navigation
- IPC/protobuf contracts
- archive/import/export assumptions
- run, sample, parameter, code, and provenance boundaries

Likely reusable areas:

- Rust/Python/frontend build infrastructure
- Arrow payload IO concepts
- append-only record and semantic manifest ideas
- chart rendering after DTO/model changes
- Tauri packaging and web frontend stack
- testing, release, and documentation infrastructure

## First Engineering Slice

The first slice should prove the new model end to end:

1. Create or open one data library.
2. Create or select a sample/session when known, or explicitly run without one.
3. Set active sample/session context from the desktop UI or Python prelude when
   appropriate.
4. Start an interactive experiment from Python.
5. Write one or more dataset artifacts through experiment-scoped handles.
6. Browse the experiment and datasets in the desktop/web UI.
7. Reopen a dataset from Python by stable ID.
8. Attach or correct sample/session context after the run when needed.
9. Record actor, code summary, run note, quality, and sample/session links.

This slice intentionally breaks old workspace/dataset assumptions where they
conflict with the v0.2 model.

## ADRs Needed Before Durable Implementation

Create ADRs before committing durable storage, API, or IPC contracts for:

- data library versus workspace public model
- sample and sample-session identity
- active sample/session context and attach-later correction policy
- general Artifact versus DatasetArtifact boundary
- dataset artifact and provenance model
- experiment-scoped dataset writer lifecycle
- minimal device adapter/capability boundary for future LabRAD replacement
- actor/auth boundary for local and remote access
- optional managed experiment desired-device-state boundary
- storage compatibility and migration policy for pre-v0.2 workspaces
- desktop web architecture and remote UI access

## Design Constraints

- Keep the Python experiment path simple.
- Keep datasets directly openable from Python and the desktop UI.
- Keep sample/session context easy to set as an active default, but do not block
  quick experiments when the context is unknown.
- Keep dataset semantics dataset-local.
- Keep sample/session/run/parameter/provenance out of dataset names.
- Keep notes and tags mostly on sample, session, experiment, analysis, and
  calibration records.
- Keep advanced execution concepts optional until users need retry, resume,
  dry-run, or calibration automation.
- Keep local-first and single-owner assumptions in the product, while leaving
  actor and remote-access hooks in the architecture.
