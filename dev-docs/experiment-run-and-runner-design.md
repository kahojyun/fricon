# Experiment Run And Runner Design Proposal

## Status

Proposed future product direction.

This is not current behavior. Do not implement or document behavior from this
proposal as user-facing functionality until the relevant dataset semantic,
parameter, storage, IPC, Python API, desktop UI, and migration work has landed.

## Purpose

Define the first durable product boundary between scientific experiment records
and generic script execution in Fricon.

The design should help users answer:

- Which scientific experiment attempt produced these datasets?
- Which immutable parameter state was used for that attempt?
- Which script executions actually ran?
- Which execution failed, retried, or continued a partial write?
- Which records in a dataset were appended by which execution attempt?
- Which devices or local resources forced tasks to run sequentially?

The proposal keeps the current route dataset-first, Python-led, and local-first.
Python scripts remain the first execution entry point. The desktop UI may
browse, inspect, retry, continue, and summarize work, but it should not become
the primary experiment execution engine before the Python-led model is clear.

## Classification

Classification: `after dataset semantics`.

ADR need: create an ADR before implementation commits to durable identifiers,
storage shape, retry/resume semantics, dataset append provenance, runner
contracts, or resource lease behavior.

The record-centric first slice depends on:

- durable dataset semantics and append-order identity
- minimal parameter snapshot binding
- a policy for immutable facts versus correction or audit events

Future runner implementation also depends on:

- a local execution service that can coordinate workspace state and script
  processes

## Settled PO Decisions For V1

The first experiment-run product slice should be record-centric.

V1 should promise:

- record a scientific experiment attempt
- bind one immutable effective parameter snapshot
- link produced datasets
- expose notes, quality state, and retry or continuation history
- keep script execution details available as debugging context

V1 should not promise a full generic runner implementation. The queue, script
run, resource requirement, and resource lease model should be documented now so
the experiment model does not block future execution work, but implementation
can start with run records and dataset provenance.

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
- code and environment reproducibility should start as passive summary metadata,
  not managed Git, `uv`, or `pixi` history
- managed submitted runs should start from importable Python functions or module
  entry points, not notebook conversion
- Fricon should provide a guided extraction path from notebook or ad hoc script
  experiments to managed templates
- default run names should use a template or script label plus timestamp
- notes should be easy to add, but Fricon should not interrupt run start or
  completion with required note prompts
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
ExperimentRun -> Dataset
```

One experiment run can produce multiple datasets. Retry and resume details should
not force users to understand execution internals when they only need to browse
experiment outputs.

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
- AI actions that mutate workspace state without review and auditability
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

| Term                  | Meaning                                                                 |
| --------------------- | ----------------------------------------------------------------------- |
| `ExperimentRun`       | One scientific experiment attempt.                                      |
| `TaskQueueEntry`      | One queued executable unit before it starts.                            |
| `ScriptRun`           | One actual script execution attempt.                                    |
| `Dataset`             | One data artifact with dataset-local semantics.                         |
| `DatasetWriteSession` | One append contribution from a script run to a dataset.                 |
| `ResourceRequirement` | A declared need for a named local resource before execution can start.  |
| `ResourceLease`       | A granted shared or exclusive hold on a named local resource.           |

## Identifier Rules

### Experiment Run Identity

Normally, every real experiment attempt gets a new `experiment_run_id`, even if
the script, parameters, sample, device, and expected outputs are identical.

Reuse an existing `experiment_run_id` only for an interrupted continuation or
retry that the user or system treats as the same scientific attempt.

Parameter changes should normally require a new experiment run because the
effective parameter snapshot is part of the experiment-run facts.

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

Dataset identity remains dataset-local. A dataset is not owned by a script run.
It may be produced under an experiment run and receive appended records from one
or more script runs through write sessions.

## Relationships

The record-centric v1 subset is:

```text
ExperimentRun 1 -> many Dataset
ExperimentRun 1 -> 1 EffectiveParameterSnapshot
ExperimentRun 1 -> many DatasetWriteSession
```

When the generic runner is implemented, the fuller relationship model becomes:

```text
ExperimentRun 1 -> many Dataset
ExperimentRun 1 -> many TaskQueueEntry
ExperimentRun 1 -> many ScriptRun
ExperimentRun 1 -> 1 EffectiveParameterSnapshot

TaskQueueEntry 1   -> 0/1 ScriptRun
ScriptRun      1   -> many DatasetWriteSession
Dataset        1   -> many DatasetWriteSession
```

Notes:

- A task may be canceled before it creates a script run.
- A script run may fail before writing any dataset records.
- A dataset may have multiple write sessions when retry or continuation appends
  to the same logical dataset.
- Before generic runner implementation, a write session may record only passive
  execution metadata instead of a durable `script_run_id`.
- Imported, processed, or simulation datasets may have different owning
  provenance in future designs.

## ExperimentRun

### Owns

`ExperimentRun` owns the scientific attempt record:

- identity and display label
- lifecycle state
- effective parameter snapshot reference
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

V1 should use one effective immutable parameter snapshot per experiment run:

```text
ExperimentRun -> EffectiveParameterSnapshot
```

If the user starts from a mutable profile or ref, Fricon should resolve it before
the experiment starts and record the resolved snapshot ID.

Retry or continuation within the same experiment run should reuse the same
snapshot. If parameter values change, the user should normally start a new
experiment run.

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

The first implementation does not need a full workspace event timeline, but the
model should preserve room for it.

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

This split reflects common scientific Python practice:

- users often start in ad hoc notebooks or scripts because iteration is fast
- repeated measurements are later extracted into reusable Python functions or
  modules
- old approaches often pass save paths, experiment names, and local parameter
  values manually, then store important parameters in dataset metadata
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
with ws.experiment_run("cooldown sweep", params="main") as run:
    with run.dataset("s21") as ds:
        ds.write(freq=..., s21=...)
```

Possible managed template shape:

```python
def cooldown_sweep(ctx, params):
    with ctx.dataset("s21") as ds:
        ds.write(freq=..., s21=...)
```

Managed execution can be introduced separately:

```python
task = ws.submit_experiment(cooldown_sweep, params="main")
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

Re-run snippets, export/report snippets, and managed-template submission
snippets are useful later, but read snippets best support the current
dataset-first and Python-led route.

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

## Future Scope

This proposal intentionally leaves the following to later focused designs:

- generic runner, task queue, script-run, and resource-lease implementation
- managed submitted run execution and template registry behavior
- workflow definitions, workflow runs, and step dependency semantics
- calibration proposal, validation, and promotion flows
- analysis run taxonomy for derived datasets
- device identity, apply plans, readback verification, and instrument snapshots
- complete workspace event timeline and audit export behavior
- AI-assisted suggestions, approvals, and accepted/rejected mutation records
- distributed execution or remote clients

## Open Questions

- How should queue priority interact with already-running resource leases?
- Should resource lease failures block a task, fail it, or leave it queued?
- What facts must survive dataset archive export and import?

## Related Documents

- `dev-docs/project-intent.md`
- `dev-docs/roadmap.md`
- `dev-docs/dataset-semantic-architecture-proposal.md`
- `dev-docs/parameter-management-design.md`
- `dev-docs/future-concepts.md`
