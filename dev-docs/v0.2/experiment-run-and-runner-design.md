# Experiment Run And Runner Design Proposal

## Status

Supporting future experiment and runner proposal for the v0.2 reset.

This is not current behavior. Do not implement or document behavior from this
proposal as user-facing functionality until the relevant dataset semantic,
parameter, storage, IPC, Python API, desktop UI, and migration work has landed.

Read `README.md` and `design.md` first. This long proposal is supporting
detail for experiment execution, retry/resume, and runner boundaries; it is not
the canonical v0.2 entry point.

## Purpose

Define the first durable product boundary between scientific experiment records
and generic script execution in Fricon.

The design should help users answer:

- Which scientific experiment attempt produced these datasets?
- Which later analysis, import, simulation, or calibration activity consumed or
  produced these datasets?
- Which immutable parameter state was used for that attempt?
- Which script executions actually ran?
- Which execution failed, retried, or continued a partial write?
- Which records in a dataset were appended by which execution attempt?
- Which devices or local resources forced tasks to run sequentially?

This proposal was written against the current dataset-first, Python-led, and
local-first route. Under the v0.2 reset, reconcile the details here with the
broader data-library, sample/session, and activity-provenance model in
`design.md`. Python scripts remain the first execution entry point. The desktop
UI may browse, inspect, retry, continue, and summarize work, but it should not
become the primary experiment execution engine before the Python-led model is
clear.

## Classification

Classification: `after dataset semantics`.

ADR need: create an ADR before implementation commits to durable identifiers,
storage shape, retry/resume semantics, dataset append provenance, runner
contracts, or resource lease behavior.

The record-centric first slice depends on:

- durable dataset semantics and append-order identity
- a policy for immutable facts versus correction or audit events

Parameter-aware runs also depend on:

- minimal parameter snapshot binding

Future runner implementation also depends on:

- a local execution service that can coordinate data-library state and script
  processes

## Settled PO Decisions For V1

The first experiment-run product slice should be record-centric.

V1 should promise:

- record a scientific experiment attempt
- optionally link an immutable effective parameter snapshot when available
- preserve user-provided JSON metadata for migration from existing experiment
  code
- attach supporting files or artifacts when users have no structured Fricon
  concept for that information yet
- link produced datasets
- expose notes, quality state, and retry or continuation history
- keep script execution details available as debugging context

V1 should not promise a full generic runner implementation. The queue, script
run, resource requirement, and resource lease model should be documented now so
the experiment model does not block future execution work, but implementation
can start with run records and dataset provenance.

V1 should also be sufficient to replace a simple LabRAD Grapher/Data Vault style
experiment logger for new measurement work. This means users can record new
experiments into Fricon, inspect the resulting datasets in the desktop UI, keep
run-level context beside the data, and reopen outputs from Python without
continuing to depend on the old logger.

V1 LabRAD-style replacement does not include bulk migration or full browsing of
legacy LabRAD/Data Vault history. Importing old history can be a follow-up
migration feature once the new run, dataset, and metadata boundaries are stable.

Settled user-facing policies:

- experiment submission should support two product modes:
  interactive runs and managed submitted runs
- every real experiment attempt gets a new `experiment_run_id`
- interrupted continuation may reuse an existing `experiment_run_id`
- continuation should require explicit user or API intent
- every execution retry gets a new `script_run_id` when script-run records
  exist
- retry may append to an existing dataset only through explicit continuation
  and compatibility checks
- runner details are secondary UI information under execution history or
  troubleshooting
- analysis scripts are future derived-dataset provenance, not core
  `ExperimentRun` v1 behavior
- dataset-only creation remains valid and should create unassigned datasets
  unless the caller explicitly supplies an experiment context
- sample/session context is encouraged but optional; users can set active
  context from a notebook prelude or desktop UI, and attach or correct context
  later through an auditable correction path
- datasets are independent artifacts that may later be produced by experiments,
  analysis runs, import runs, simulation runs, or calibration workflows
- `Artifact` is the general provenance output concept; `Dataset` is the first
  concrete artifact subtype needed for measurement replacement
- interactive experiment parameter snapshots are optional; user-provided
  metadata can bridge existing code until the parameter system is adopted
- managed or parameter-aware runs should require a resolved immutable parameter
  snapshot when they claim strong reproducibility
- code and environment reproducibility should start as passive summary metadata,
  not managed Git, `uv`, or `pixi` history
- managed submitted runs should start from importable Python functions or module
  entry points, not notebook conversion
- Fricon should provide a guided extraction path from notebook or ad hoc script
  experiments to managed templates
- default run names should use a template or script label plus timestamp
- notes should be easy to add, but Fricon should not interrupt run start or
  completion with required note prompts
- experiment-level notes, tags, pin or favorite state, quality state, and legacy
  JSON metadata should live on the run by default
- dataset metadata should stay focused on output-local semantics, per-output
  notes, and per-output quality exceptions
- interrupted partial datasets should default to `suspect` until the user
  continues, validates, or invalidates them
- run detail views should lead with produced datasets
- provenance level should be shown as a plain label such as `Interactive` or
  `Managed`, not as a numeric confidence score
- generated snippets should prioritize reopening produced datasets from Python

The next product discussion should be dataset semantics v1, especially the
dataset facts needed to make retry append safe and explainable.

## Product Goals

### Keep Experiment Meaning Separate From Execution Mechanics

An experiment run is a scientific attempt. A script run is an execution attempt.
They have different lifecycles, identifiers, failure modes, and user meanings.

The runner should not need to know what an experiment is. It should execute
generic tasks, capture execution facts, coordinate local resources, and report
status. Experiment, workflow, calibration, and analysis concepts should live in
higher layers that reference runner records.

### Preserve A Simple Experiment To Dataset Relationship

The user-facing scientific model should stay close to:

```text
ExperimentRun -> produced Dataset
```

One experiment run can produce multiple datasets. This is a link, not exclusive
ownership: datasets remain independently addressable artifacts and future
analysis, import, simulation, and calibration activities may also produce or
consume datasets. Retry and resume details should not force users to understand
execution internals when they only need to browse experiment outputs.

Dataset-only creation should remain a lower-level path. It is useful for
imports, standalone tables, tests, transitional scripts, and future derived
outputs that are not naturally part of a measurement attempt. Dataset writers
should link to an experiment run only through an explicit run context or
ownership argument, not by silently creating synthetic experiments at the
storage layer.

### Keep Activity Provenance Graph-Shaped

Future analysis, import, simulation, calibration, and workflow systems should
share a producer/consumer provenance pattern:

```text
concrete run record consumes inputs and produces outputs
```

Concrete user-facing run types should remain meaningful:

- `ExperimentRun` for measurement attempts
- `AnalysisRun` for processing, fitting, summarizing, and derived outputs
- `ImportRun` for external data ingestion
- `SimulationRun` for generated data
- `CalibrationRun` or `WorkflowRun` for coordinated multi-step automation

The shared provenance model should connect these records to datasets,
artifacts, parameter snapshots, analysis results, and parameter proposals
without forcing all activity types into the experiment model.

### Make Retry And Resume Auditable

A failed execution retry should always create a new script run record. When the
retry is a continuation of the same scientific attempt, it may reuse the same
experiment run and append to the same dataset through an explicit write-session
record.

Retries should preserve enough context to explain what failed, what continued,
which dataset records were appended, and whether the final output should be
trusted.

### Keep Scheduling Simple For The Future Runner

The future runner should use a global task queue with priority and local
resource leases. It should not become a general workflow DAG engine.

Future workflow, calibration, optimization, benchmark, and AI-assisted systems
can define dependencies and approval checkpoints above the runner. The runner
should execute tasks that are already eligible to run.

### Leave Room For Future Workflow And Device Concepts

Resource keys and execution records should be able to reference future device
identity, instrument snapshots, workflow steps, calibration steps, and audit
events. They should not require a broad hardware driver framework or workflow
engine in the first implementation.

## Non-Goals

This design should not introduce:

- multi-user scheduling, permissions, accounts, or hosted workers
- distributed locks or cluster execution
- a full DAG scheduler in the runner layer
- a requirement to implement the generic runner in the first experiment-run
  slice
- a broad hardware driver framework
- desktop-first experiment execution as the primary product model
- automatic parameter mutation after calibration
- AI actions that mutate data-library state without review and auditability
- rewriting or deleting completed measurement facts as the normal correction
  path

## Core Model

### Layers

```text
Experiment layer
  ExperimentRun
  ParameterSnapshot binding
  Dataset relationship
  DatasetWriteSession provenance

Future activity provenance layer
  AnalysisRun
  ImportRun
  SimulationRun
  RunInput
  RunOutput
  AnalysisResult
  ParameterProposal link

Future execution layer
  TaskQueueEntry
  ScriptRun
  ResourceRequirement
  ResourceLease
  logs, exit status, retry records

Future orchestration layer
  WorkflowRun
  CalibrationRun
  step dependencies
  approval checkpoints
  automation decisions
```

The execution layer should remain generic when it is implemented. Higher layers
may attach domain meaning to tasks and script runs.

### Terms

| Term                  | Meaning                                                                |
| --------------------- | ---------------------------------------------------------------------- |
| `ExperimentRun`       | One scientific experiment attempt.                                     |
| `AnalysisRun`         | Future record for processing, fitting, summarizing, or deriving data.  |
| `RunInput`            | Future provenance edge from a run to a consumed dataset, run, parameter snapshot, or artifact. |
| `RunOutput`           | Future provenance edge from a run to a produced dataset, result, proposal, or artifact. |
| `AnalysisResult`      | Future structured analysis outcome that may support reports, decisions, or parameter proposals. |
| `TaskQueueEntry`      | One queued executable unit before it starts.                           |
| `ScriptRun`           | One actual script execution attempt.                                   |
| `Dataset`             | One data artifact with dataset-local semantics.                        |
| `DatasetWriteSession` | One append contribution from a script run to a dataset.                |
| `ResourceRequirement` | A declared need for a named local resource before execution can start. |
| `ResourceLease`       | A granted shared or exclusive hold on a named local resource.          |

## Identifier Rules

### Experiment Run Identity

Normally, every real experiment attempt gets a new `experiment_run_id`, even if
the script, parameters, sample, device, and expected outputs are identical.

Reuse an existing `experiment_run_id` only for an interrupted continuation or
retry that the user or system treats as the same scientific attempt.

For parameter-aware runs, parameter changes should normally require a new
experiment run because the resolved parameter snapshot is part of the
experiment-run facts. Interactive migration runs may instead preserve
user-provided parameter metadata until the experiment adopts Fricon parameter
snapshots.

### Script Run Identity

Every execution attempt gets a new `script_run_id`, including retries and
continuations.

Script run identity is about runtime facts:

- executable entry point
- arguments
- environment summary
- start and finish times
- exit status
- logs
- resource leases
- retry or continuation links

### Dataset Identity

Dataset identity remains dataset-local. A dataset is not owned by a script run
or by an experiment run. It may be produced under an experiment run and receive
appended records from one or more script runs through write sessions. Future
analysis, import, simulation, and calibration activities may also produce or
consume datasets through provenance edges.

The dataset layer should not silently create experiment records. A dataset may
remain unassigned when created through a dataset-only API. Higher-level run,
analysis, import, or workflow APIs own the decision to link a dataset to a
producer record.

## Relationships

The record-centric v1 subset is:

```text
ExperimentRun 1 -> many produced Dataset links
ExperimentRun 0/1 -> 1 EffectiveParameterSnapshot
ExperimentRun 1 -> many DatasetWriteSession
```

When the generic runner is implemented, the fuller relationship model becomes:

```text
ExperimentRun 1 -> many produced Dataset links
ExperimentRun 1 -> many TaskQueueEntry
ExperimentRun 1 -> many ScriptRun
ExperimentRun 0/1 -> 1 EffectiveParameterSnapshot

TaskQueueEntry 1   -> 0/1 ScriptRun
ScriptRun      1   -> many DatasetWriteSession
Dataset        1   -> many DatasetWriteSession
```

Future derived-data provenance should extend the relationship model without
changing dataset identity:

```text
AnalysisRun 1 -> many RunInput links
AnalysisRun 1 -> many RunOutput links
RunInput  -> Dataset | ExperimentRun | ParameterSnapshot | Artifact
RunOutput -> Dataset | AnalysisResult | ParameterProposal | Artifact
```

Notes:

- A task may be canceled before it creates a script run.
- A script run may fail before writing any dataset records.
- A dataset may have multiple write sessions when retry or continuation appends
  to the same logical dataset.
- Before generic runner implementation, a write session may record only passive
  execution metadata instead of a durable `script_run_id`.
- Imported, processed, or simulation datasets should use the same provenance
  pattern rather than pretending every dataset is experiment-owned.

## ExperimentRun

### Owns

`ExperimentRun` owns the scientific attempt record:

- identity and display label
- lifecycle state
- optional effective parameter snapshot reference
- user-provided metadata for migration and workflow-specific context
- attachment references for supporting files or artifacts
- optional parameter source ref used to resolve the snapshot
- optional sample or specimen reference in future work
- expected or produced dataset links
- related task and script run links
- notes, tags, quality state, and correction references

### Does Not Own

`ExperimentRun` does not own:

- dataset payload storage
- dataset-local semantic interpretation
- low-level process execution
- workflow scheduling policy
- device driver behavior
- parameter registry history

### Lifecycle

The user-facing v1 outcome and quality vocabulary should stay small:

```text
completed
failed
aborted
suspect
invalidated
```

`completed`, `failed`, and `aborted` describe run outcome. `suspect` and
`invalidated` describe later quality or trust decisions. Transient internal
states such as `planned` or `running` may exist if needed, but they should not
grow into a large user-facing status taxonomy.

The key policy is that completed scientific facts should not be silently
rewritten. Later changes should use correction, invalidation, or audit records.

## Parameter Snapshot Binding

Interactive experiment runs should not require parameter management adoption.
They may record a user-provided parameter summary or custom metadata when the
experiment has not yet moved to Fricon's parameter snapshot model.

Parameter-aware and managed submitted runs should use one effective immutable
parameter snapshot per experiment run:

```text
ExperimentRun 0/1 -> EffectiveParameterSnapshot
```

If the user starts from a mutable profile or ref, Fricon should resolve it before
the parameter-aware experiment starts and record the resolved snapshot ID.

Retry or continuation within the same experiment run should reuse the same
snapshot. If parameter values change, the user should normally start a new
experiment run.

User-provided metadata is a migration bridge, not a replacement for durable
parameter snapshots. It should be searchable and visible enough to help users
migrate existing experiment code, but Fricon should not treat arbitrary metadata
as structured parameter history.

Future systems may introduce parameter bindings with roles:

```text
ExperimentRun -> ParameterBinding -> ParameterSnapshot
```

That future shape can support measurement parameters, analysis parameters,
device configuration snapshots, and multi-stage workflows without complicating
the v1 user model.

## TaskQueueEntry

`TaskQueueEntry` is the future scheduler-facing record for work that may run.

It is design guidance for the runner layer, not required for the first
record-centric experiment-run implementation.

Candidate fields:

- `task_queue_entry_id`
- `task_kind`, initially `script`
- `priority`
- `status`
- `entrypoint`
- `args`
- `environment_request`
- `resource_requirements`
- `created_by`
- `enqueued_at`
- `not_before`
- optional domain owner reference, such as `experiment_run_id`

Candidate states:

```text
queued
blocked
running
completed
failed
canceled
expired
```

The queue should decide when a task is eligible to start by checking priority,
queue state, `not_before`, cancellation, and resource availability.

The queue should not own workflow dependency meaning. Future workflow and
calibration layers should enqueue only tasks whose domain dependencies have
already been satisfied, or should wake the scheduler when a step becomes
runnable.

## ScriptRun

`ScriptRun` records one execution attempt after a queued task starts.

It should exist when Fricon owns script execution through a runner or queue.
Before that layer exists, `ExperimentRun` and `DatasetWriteSession` may keep a
passive execution summary for user-run Python scripts.

Candidate fields:

- `script_run_id`
- `task_queue_entry_id`
- `entrypoint`
- `args`
- `working_directory`
- `environment_summary`
- `code_reference`
- `status`
- `started_at`
- `finished_at`
- `exit_code`
- `failure_reason`
- `log_refs`
- `retry_of_script_run_id`
- `continuation_of_script_run_id`

Candidate states:

```text
starting
running
succeeded
failed
aborted
lost
```

`ScriptRun` should remain generic. It may be linked to experiment, workflow, or
calibration records, but the runner should not need to interpret those links to
execute a script.

Passive execution summary for the record-centric v1 slice may include:

- script path or display name
- arguments when known and safe to record
- current working directory summary
- Python version
- Fricon package version
- optional lock-file, script hash, or code reference when easy to capture

This passive summary should not imply that Fricon manages Git history,
environments, `uv`, or `pixi`.

## DatasetWriteSession

`DatasetWriteSession` is the provenance bridge between script execution and
dataset append facts.

It exists because this relationship is not strictly one-to-many:

```text
ScriptRun -> Dataset
```

A single script run can write multiple datasets, and one dataset can receive
append segments from multiple script runs during retry or continuation.

Candidate fields:

- `dataset_write_session_id`
- `dataset_id`
- optional `script_run_id`
- optional `experiment_run_id`
- `status`
- `started_at`
- `finished_at`
- `first_record_id`
- `last_record_id`
- `record_count`
- `continuation_of_dataset_write_session_id`
- `failure_reason`

Candidate states:

```text
open
completed
failed
aborted
superseded
```

### Append Policy

Retry may append to an existing dataset only when:

- the retry belongs to the same experiment run
- the user or Python API explicitly declares continuation
- the dataset schema is already frozen and unchanged
- the dataset semantics and manifest are unchanged
- append order is preserved by durable record identity
- existing rows are not overwritten or deleted
- the dataset has not been finalized in a way that forbids continuation

Retry should create a new dataset when:

- parameters changed
- sample, device configuration, or scientific setup changed
- the old partial result should remain a meaningful failed measurement
- the retry would change dataset meaning or scan semantics
- the user chooses a new scientific attempt instead of continuation

## Resource Requirements And Leases

Device and execution conflicts should be expressed as named local resources
rather than hard-coded device concepts.

This section belongs to the future runner implementation. It should shape the
model now, but resource leases do not need to ship with record-centric
`ExperimentRun` v1.

Candidate resource requirement fields:

- `task_queue_entry_id`
- `resource_key`
- `mode`: `exclusive` or `shared`
- `purpose`

Candidate lease fields:

- `resource_lease_id`
- `resource_key`
- `holder_script_run_id`
- `mode`
- `status`
- `acquired_at`
- `released_at`

Example resource keys:

```text
device:qcm_1
device:lockin_1
workspace:python-env
```

The first runner implementation can implement leases with local workspace server
coordination and SQLite transactions. It should not require distributed locks.

The scheduler should start a task only after all required leases can be acquired
or reserved according to the local policy.

## Retry And Continuation

Retry rules:

- when the runner exists, every retry creates a new task queue entry
- when script-run records exist, every retry execution creates a new script run
- the new script run records `retry_of_script_run_id` or
  `continuation_of_script_run_id`
- continuation may append to an existing dataset through a new write session
- retry with changed scientific facts should create a new experiment run

Suggested user-facing policy:

```text
Retry same experiment:
  use when execution failed or communication dropped and the same scientific
  attempt should continue

Start new experiment:
  use when parameters, setup, sample, device state, or scientific meaning
  changed
```

The desktop UI or Python API should require explicit user intent before a retry
continues an existing experiment run. The system may suggest continuation when
state is compatible, but it should not silently infer it.

## Scheduling Model

When the runner is implemented, it should use:

```text
global task queue + priority + resource leases
```

It should not model a general dependency graph on script runs.

This keeps the runner small and leaves domain dependency semantics to future
workflow and calibration layers.

Future orchestration can use:

```text
WorkflowRunStep -> TaskQueueEntry -> ScriptRun
CalibrationStep -> TaskQueueEntry -> ScriptRun
```

The orchestration layer can decide when a step becomes runnable based on
dependencies, approvals, validation results, or manual overrides. The scheduler
only receives runnable or blocked queue entries and enforces priority and
resource availability.

## Provenance And Audit Boundary

The first implementation does not need a full data-library event timeline, but
the model should preserve room for it.

Actions that likely need durable audit or event records:

- experiment run created, completed, failed, aborted, invalidated, or corrected
- task enqueued, canceled, retried, or manually reprioritized
- script run failed, aborted, lost, or continued
- dataset reopened or continued after interruption
- dataset write session failed or appended records after retry
- resource lease acquisition failed or was manually overridden
- future workflow, calibration, device, or AI actions that mutate state

Audit records should summarize user-visible mutations without exposing internal
storage paths or protocol details as the user model.

## Experiment Submission UX And Provenance Levels

Fricon should support two experiment submission modes with different provenance
promises.

This split reflects common scientific Python practice and migration from
external experiment record systems such as LabRAD Data Vault, lab-local data
vaults, ad hoc HDF5/JSON/CSV folders, and notebook-managed output directories:

- users often start in ad hoc notebooks or scripts because iteration is fast
- repeated measurements are later extracted into reusable Python functions or
  modules
- existing tools often pass save paths, experiment names, and local parameter
  values manually, then store important experiment context on the recorded data
  object
- Fricon should improve traceability without making early exploration
  cumbersome

The two modes should share the same experiment and dataset concepts, but they
should not pretend to provide the same reproducibility guarantees.

### Interactive Run

Interactive runs are user-controlled runs from notebooks, REPL sessions, or ad
hoc scripts.

Use this mode for:

- exploratory measurement
- fast iteration and debugging
- one-off experiments
- gradual migration from existing notebooks or local scripts

In this mode, the user script controls the experiment lifecycle directly through
the Python API.

Fricon should record:

- `ExperimentRun`
- effective parameter snapshot or user-provided parameter summary
- custom JSON metadata for migration from existing scripts
- attachment references for supporting files or artifacts
- produced datasets
- dataset write sessions when datasets are appended
- notes, quality state, and explicit continuation decisions
- passive code and environment summary when available

Fricon should not over-promise:

- complete notebook state capture
- complete imported code history
- automatic retry
- resource leases
- queue priority
- complete stdout or stderr capture
- replayable execution entry point

This mode is still a first-class product path. It should make lightweight
experiment records useful without forcing users to adopt a runner before their
experiment has stabilized.

Existing systems often store experiment-level context on individual datasets.
Interactive runs should provide a better landing point for that context:

- experiment-level labels, tags, and quality state should live on the run when
  they describe the whole attempt
- dataset metadata should remain for dataset-local semantics and per-output
  details
- arbitrary JSON metadata and attachments can preserve legacy context while
  users gradually move stable concepts into parameter snapshots, run fields, or
  future sample/device records

### Managed Submitted Run

Managed submitted runs are system-executed runs from an importable Python
function or module entry point.

Use this mode for:

- repeated measurements
- important data collection
- experiments that need stronger provenance
- runs that need retry or continuation support
- future resource leases, queueing, workflow, calibration, or automation

The first managed template contract should be an importable Python function or
module entry point that receives a Fricon execution context and resolved
parameters. The exact decorator, context object, and submission API should be
designed later.

When the runner exists, Fricon can own:

- task queue entry
- script run record
- resolved immutable parameter snapshot
- logs, exit status, and failure reason
- resource requirements and leases
- retry and continuation records
- dataset write sessions
- stronger code and environment references

Managed submitted runs are the bridge toward future workflow and calibration
automation, but they should not require a full workflow engine.

Future managed runs may use an optional declarative experiment plan model for
experiments that need retry, resume, dry-run, dummy-device execution, device
state preview, readback verification, or automatic calibration. In that model,
the experiment code declares how parameter snapshots, run-local inputs, and
scan points resolve into desired device state and dataset outputs.

This should not become mandatory for simple interactive runs. Existing
imperative Python code remains a valid way to record experiments and datasets;
the tradeoff is that advanced managed features may require the declarative plan
or explicit advanced lifecycle hooks.

### Guided Extraction Path

Fricon should help users move from interactive exploration to managed submitted
runs without treating notebooks as a mistake.

The product should favor guided extraction over automatic conversion:

```text
notebook or ad hoc script
  -> interactive experiment run
  -> reusable Python function or module
  -> managed submitted run
  -> future workflow or calibration template
```

Possible aids:

- generated Python snippets from an interactive run
- template skeletons that use the same dataset and parameter APIs
- desktop links from a run detail view back to relevant Python snippets
- validation that important parameters are captured as run parameters or
  parameter snapshots instead of dataset metadata
- warnings when a managed template writes important experiment context only into
  dataset-local metadata

Important experiment facts should move toward run records, parameter snapshots,
and provenance links. Dataset metadata should remain focused on dataset-local
semantics such as columns, axes, units, display hints, and scan interpretation.

## UI And Python API Direction

### Python API

The first public API should keep simple data collection scripts ergonomic.
Users should not need to construct raw runner records for common measurement
workflows.

Possible interactive shape:

```python
lib = fricon.library()
lib.use_context(sample="sample-a", session="cooldown-2026-05")

with lib.experiment("cooldown sweep", params="main") as run:
    s21 = run.dataset("s21")
    noise = run.dataset("noise")

    for freq in freqs:
        s21.write(freq=freq, s21=measure_s21(freq))
        noise.write(freq=freq, noise=measure_noise(freq))
```

The active sample/session context is a convenience for the common notebook
prelude pattern, not a hidden global fact. The experiment record should store
the resolved context IDs when present. A quick experiment without context should
still be valid and visibly missing context.

The experiment context should own default dataset writer finalization for
datasets opened through the run. On normal run exit, open produced datasets are
finished. On exceptional run exit, open produced datasets are aborted or marked
suspect according to the settled lifecycle policy. A dataset writer may still
offer explicit `finish()` or `abort()` for multi-output runs where one output
ends earlier or fails independently.

Nested dataset context managers may remain available for advanced explicit
lifecycle control:

```python
lib = fricon.library()

with lib.experiment("cooldown sweep", params="main") as run:
    with run.dataset("s21") as s21:
        s21.write(freq=..., s21=...)
```

Public V1 examples should prefer the flatter experiment-scoped writer form when
it is sufficient.

Possible managed template shape:

```python
def cooldown_sweep(ctx, params):
    s21 = ctx.dataset("s21")
    for freq in freqs:
        s21.write(freq=freq, s21=measure_s21(freq))
```

Managed execution can be introduced separately:

```python
lib = fricon.library()
task = lib.submit_experiment(cooldown_sweep, params="main")
```

Runner-managed execution should remain future scope until the record-centric
experiment model is stable.

### Desktop UI

Desktop views should emphasize:

- experiment outputs and quality state
- produced datasets
- parameter snapshot used by the experiment
- provenance level: interactive or managed
- script execution history for debugging
- retry or continue actions when safe
- resource conflicts and queue status when tasks are managed by Fricon
- guided extraction from interactive runs to reusable templates

Execution details should be available without becoming the primary concept users
must understand to browse experiment results.

Runner concepts such as script runs should be shown as execution history or
troubleshooting detail, not as a peer navigation object beside experiments and
datasets in v1.

## Quality Of Life Defaults

Experiment-run UX should optimize for low-friction scientific work while still
making provenance visible.

### Naming

Fricon should generate a useful default run name instead of requiring users to
name every run.

The default should combine a template or script label with a timestamp:

```text
cooldown_sweep 2026-04-30 14:32
```

Users should be able to rename the run later. Auto-generated names should be
descriptive enough for recent-run lists without becoming a permanent source of
scientific meaning.

### Notes

Notes should be optional and non-blocking.

Fricon should not interrupt experiment start or completion with a required notes
prompt. Users should be able to add or edit notes from the run detail view, and
future UI can make notes easy to find without turning them into mandatory
workflow steps.

### Interrupted Partial Data

When interruption leaves a partial dataset, Fricon should preserve the data and
default the affected dataset or run quality to `suspect`.

The recovery UI should make the next actions explicit:

- continue same experiment
- start new experiment
- keep suspect
- mark valid
- invalidate

This keeps failure visible without discarding useful partial data or implying
that an interrupted dataset is fully trusted.

### Run Detail Priority

The top of the experiment-run detail view should lead with produced datasets.

Recommended information order:

```text
datasets
quality state
parameter snapshot
provenance level
notes
execution history
```

Users usually inspect outputs first. Parameter and execution provenance should
be close enough to explain those outputs without making debugging details the
primary browsing model.

### Provenance Labels

Runs should display a plain provenance label:

```text
Interactive
Managed
```

Avoid numeric reproducibility or provenance scores. They imply precision Fricon
cannot honestly provide, especially for notebook and ad hoc script workflows.

### Python Snippets

Generated snippets should first help users reopen produced datasets from Python.

Re-run snippets, report snippets, and managed-template submission snippets are
useful later. Read snippets and experiment export snippets best support the
current Python-led route and the v0.2 goal of keeping datasets directly
reopenable, including outside the source data library.

## High-Value User Stories

### Record An Interactive Experiment From A Notebook

As a researcher iterating in a notebook, I want to create an interactive
experiment run with a small amount of Python code so that Fricon records the
datasets, notes, quality state, and basic context for my exploratory
measurement.

Acceptance notes:

- parameter snapshot is optional
- user-provided JSON metadata can preserve local parameter dictionaries or
  legacy context
- attachments can reference supporting files or artifacts
- produced datasets are linked to the run
- provenance label is `Interactive`

### Migrate From An External Data Vault

As a user with an existing LabRAD Data Vault, lab-local data vault, or ad hoc
file-based experiment logger, I want to bring experiment-level context into
Fricon without immediately adopting the parameter registry or managed runner so
that migration can start with low risk.

Acceptance notes:

- V1 migration means new work can move to Fricon without depending on the old
  logger
- run metadata can store legacy JSON context
- attachments can preserve external configuration files, screenshots, or logs
- datasets remain readable as outputs of the run
- Fricon does not treat arbitrary JSON metadata as structured parameter history
- users can gradually move stable concepts into parameter snapshots, run
  fields, or future sample/device records
- bulk import or full browsing of legacy data-vault history is future scope

### Recover From An Interrupted Measurement

As a user whose measurement was interrupted by a device communication failure, I
want Fricon to preserve the partial data and offer an explicit continuation path
so that I can recover useful data without hiding the interruption.

Acceptance notes:

- partial datasets default to `suspect`
- the recovery UI offers continue same experiment and start new experiment
- continuing requires explicit user or API intent
- continuation creates a new dataset write session
- users can later keep suspect, mark valid, or invalidate the result

### Inspect A Run By Its Outputs

As a user analyzing results, I want the experiment-run detail page to lead with
produced datasets so that I can quickly reopen data while still seeing
parameters, metadata, notes, and execution history nearby.

Acceptance notes:

- produced datasets are the primary detail-view content
- Python read snippets are available for datasets
- export snippets are available for experiment-centered portable bundles
- run quality state is visible
- provenance label is visible
- parameter snapshot or migration metadata summary is visible
- execution details remain secondary debugging context

### Extract A Managed Template From Repeated Interactive Work

As a user who has repeated a notebook or ad hoc script several times, I want
guidance for extracting the stable measurement into an importable Python
function so that future runs can use stronger managed provenance.

Acceptance notes:

- Fricon provides snippets or skeletons that use the same run and dataset APIs
- Fricon does not attempt automatic notebook conversion
- the guidance points out metadata that is still arbitrary JSON
- the user can move repeated parameters toward parameter snapshots
- the resulting template can become a managed submitted run later

### Submit A Managed Experiment For Stronger Provenance

As a user running an important repeated measurement, I want to submit an
importable Python function to Fricon so that the system can eventually own logs,
status, retry, resource leases, and stronger code/environment references.

Acceptance notes:

- managed runs use an importable function or module entry point
- parameter-aware managed runs resolve an immutable parameter snapshot
- provenance label is `Managed`
- future runner implementation can create task and script-run records
- produced datasets are linked through dataset write sessions

## Settled V1 Boundary: Dataset Vs Experiment Metadata

This proposal settles the default V1 ownership boundary for new interactive
experiment workflows. Experiment-level organization should live on the run, not
be duplicated onto every produced dataset. Dataset metadata remains available
for output-specific meaning and exceptions.

V1 ownership direction:

```text
ExperimentRun metadata
  run-level tags
  run-level favorite or pin state
  run-level quality state
  notes about the scientific attempt
  sample/context references
  migration JSON metadata
  attachments

Dataset metadata
  dataset-local semantics
  column roles, axes, units, labels, and display hints
  output-specific notes or labels
  per-output quality when a single dataset differs from the whole run
```

Non-table outputs should be modeled as artifacts rather than squeezed into
dataset metadata. Attachments, logs, reports, figures, code summaries, waveform
files, and future device snapshots should share provenance links with datasets
without pretending to be table-shaped data.

Remaining questions to settle later:

- How should dataset list views behave when most organization happens at the
  experiment-run level?
- How should imports from external data vaults map legacy metadata into run
  metadata versus dataset metadata?
- Should run-level quality propagate to produced datasets, or should the two
  remain independent?

## Future Boundary Discussion: Experiment-First User Model

Interactive experiment runs may change the primary user object from dataset to
experiment.

The current Fricon user flow is dataset-first:

```python
with ws.dataset("s21") as ds:
    ds.write(...)
```

After interactive experiment runs exist, the more natural measurement flow may
be experiment-first:

```python
with ws.experiment("cooldown sweep") as exp:
    s21 = exp.dataset("s21")
    s21.write(...)
```

This proposal does not require removing dataset-only creation. Dataset-only
paths remain useful for imports, quick table capture, tests, low-level API
work, and data that is not naturally part of a measurement attempt. But product
documentation, examples, and desktop navigation may need to move toward
experiment-first workflows once runs are first-class.

Likely long-term shape:

```text
Experiment-first path
  recommended path for scientific measurement
  data library -> experiment run -> artifact outputs

Dataset-only path
  lower-level path for imports, standalone tables, and transitional workflows
  data library -> dataset artifact
```

Questions to settle later:

- Should new user-facing tutorials start from `ExperimentRun` instead of
  dataset creation once the run API exists?
- Should `ws.dataset(...)` remain a first-class public convenience path, or be
  positioned as a lower-level dataset-only path?
- How should dataset writers discover or inherit the current experiment context?
- How should active sample/session context be displayed so users notice when it
  is unset or stale?
- Should imported or legacy datasets create synthetic import runs, remain
  dataset-only, or support both?
- Should the desktop home view be run-first, dataset-first, or split by task?
- How should search, tags, favorites, quality, and recent activity behave when
  users primarily act on experiment runs?
- How should unassigned datasets be surfaced so they are useful without
  undermining the experiment-first measurement path?

## Next Product Work

The next product discussion should focus on dataset semantics v1 because
experiment retry and dataset continuation depend on stable dataset facts.

Questions to settle there:

- durable append-order identity
- dataset finalized versus continuable state
- schema and manifest compatibility checks for continuation
- column roles, axes, scan semantics, units, labels, and display hints
- how partial writes and failed write sessions affect dataset quality state

After that, the next product slices should be:

- experiment run detail view
- parameter snapshot binding and display
- retry or continue UX
- interactive versus managed provenance indicators
- guided extraction from notebook or ad hoc script to importable template
- run notes and quality flags
- dataset versus experiment-run metadata ownership
- experiment-first user model and dataset-only fallback path

## Tech Lead Discussion Items

Before implementation, discuss:

- whether current dataset storage can safely support continuation append
- how `DatasetWriteSession` state stays consistent with chunk writes
- crash recovery when the server, Python script, or device communication dies
- atomicity across experiment-run state, dataset state, and write-session state
- which passive code and environment fields can be captured reliably
- whether to reserve storage for future `TaskQueueEntry` and `ScriptRun` now or
  defer it until runner implementation
- which states are user-facing contracts and which remain internal storage
  states
- what can be passively captured from notebooks or ad hoc scripts without
  creating false reproducibility claims
- how an importable function template should be loaded, invoked, and isolated
  when managed submission is implemented
- how to validate that managed templates record important experiment context in
  run or parameter records rather than dataset-local metadata
- how an experiment context manager should coordinate dataset writer ownership,
  default finalization, early per-output finish or abort, and error handling
- how top-level dataset writer APIs represent unassigned datasets and optional
  explicit run ownership

## Future Scope

This proposal intentionally leaves the following to later focused designs:

- generic runner, task queue, script-run, and resource-lease implementation
- managed submitted run execution and template registry behavior
- workflow definitions, workflow runs, and step dependency semantics
- calibration proposal, validation, and promotion flows
- analysis run taxonomy and shared input/output provenance for derived datasets
- device identity, apply plans, readback verification, and instrument snapshots
- complete data-library event timeline and audit export behavior
- AI-assisted suggestions, approvals, and accepted/rejected mutation records
- distributed execution or remote clients

## Open Questions

- How should queue priority interact with already-running resource leases?
- Should resource lease failures block a task, fail it, or leave it queued?
- What facts must survive dataset archive export and import?
- What facts must survive experiment-centered portable export when the user
  opens the bundle directly without importing it?

## Related Documents

- `dev-docs/project-intent.md`
- `dev-docs/roadmap.md`
- `dev-docs/dataset-semantic-architecture-proposal.md`
- `dev-docs/v0.2/parameter-management-design.md`
- `dev-docs/v0.2/future-concepts.md`
