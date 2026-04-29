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
Python scripts remain the first execution entry point. The desktop UI may browse,
inspect, enqueue, retry, and summarize work, but it should not become the primary
experiment execution engine before the Python-led model is clear.

## Classification

Classification: `after dataset semantics`.

ADR need: create an ADR before implementation commits to durable identifiers,
storage shape, retry/resume semantics, dataset append provenance, runner
contracts, or resource lease behavior.

This proposal depends on:

- durable dataset semantics and append-order identity
- minimal parameter snapshot binding
- a policy for immutable facts versus correction or audit events
- a local execution service that can coordinate workspace state and script
  processes

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

### Keep Scheduling Simple In V1

The first runner should use a global task queue with priority and local resource
leases. It should not become a general workflow DAG engine.

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

Execution layer
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

The execution layer should remain generic. Higher layers may attach domain
meaning to tasks and script runs.

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

The preferred v1 relationship model is:

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

Candidate states:

```text
planned
running
completed
failed
aborted
continued
invalidated
```

State names should be narrowed before implementation. The key policy is that
completed scientific facts should not be silently rewritten. Later changes
should use correction, invalidation, or audit records.

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

`TaskQueueEntry` is the scheduler-facing record for work that may run.

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
- `script_run_id`
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

- the retry belongs to the same experiment run or is explicitly declared as a
  continuation
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

V1 can implement leases with local workspace server coordination and SQLite
transactions. It should not require distributed locks.

The scheduler should start a task only after all required leases can be acquired
or reserved according to the local policy.

## Retry And Continuation

Retry rules:

- every retry creates a new task queue entry
- every retry execution creates a new script run
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

The desktop UI may eventually ask the user to confirm whether a retry should
continue an existing experiment run or start a new one when the answer is not
obvious.

## Scheduling Model

The v1 runner should use:

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

## UI And Python API Direction

### Python API

The first public API should keep simple data collection scripts ergonomic.
Users should not need to construct raw runner records for common measurement
workflows.

Possible future shape:

```python
with ws.experiment_run("cooldown sweep", params="main") as run:
    with run.dataset("s21") as ds:
        ds.write(freq=..., s21=...)
```

Runner-managed execution can be introduced separately:

```python
task = ws.queue_script("measure.py", args=["--sample", "A"])
```

The exact API should be designed after storage and lifecycle policy are settled.

### Desktop UI

Desktop views should emphasize:

- experiment outputs and quality state
- produced datasets
- parameter snapshot used by the experiment
- script execution history for debugging
- retry or continue actions when safe
- resource conflicts and queue status when tasks are managed by Fricon

Execution details should be available without becoming the primary concept users
must understand to browse experiment results.

## Future Scope

This proposal intentionally leaves the following to later focused designs:

- workflow definitions, workflow runs, and step dependency semantics
- calibration proposal, validation, and promotion flows
- analysis run taxonomy for derived datasets
- device identity, apply plans, readback verification, and instrument snapshots
- complete workspace event timeline and audit export behavior
- AI-assisted suggestions, approvals, and accepted/rejected mutation records
- distributed execution or remote clients

## Open Questions

- Which lifecycle state names should become durable public or storage-facing
  terms?
- Should continuation of an experiment run require explicit user confirmation?
- When should a partial dataset become `suspect`, `failed`, or still valid?
- What minimal code reference should be captured without surprising users or
  turning Fricon into a Git client?
- Which environment facts should be captured by default?
- Should analysis scripts attach to an existing experiment run, create separate
  analysis runs, or both?
- How should queue priority interact with already-running resource leases?
- Should resource lease failures block a task, fail it, or leave it queued?
- What facts must survive dataset archive export and import?

## Related Documents

- `dev-docs/project-intent.md`
- `dev-docs/roadmap.md`
- `dev-docs/dataset-semantic-architecture-proposal.md`
- `dev-docs/parameter-management-design.md`
- `dev-docs/future-concepts.md`
