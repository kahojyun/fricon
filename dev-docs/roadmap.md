# Product Roadmap

## Status

AI-oriented product planning note. This file is intentionally lightweight and
should not replace issues, pull requests, implementation plans, or release
notes.

## Purpose

Use this roadmap to orient low-context human or AI contributors before choosing
implementation work. It summarizes project direction at a coarse level and
links to the documents that contain detailed policy, plans, or current
implementation facts.

Keep this file short. If an item needs step-by-step execution details, put those
details in a focused implementation plan and link it from here.

## Relationship To Current State

Current implemented user-facing behavior is still centered on:

- local workspaces
- dataset creation, write, list, read, tag, favorite, and delete flows
- Arrow-compatible table payloads with schema inferred from the first row
- Python API, CLI, server process, and desktop dataset explorer surfaces

Use `docs/concepts.md`, `docs/dataset.md`, and
`dev-docs/current-storage-notes.md` as the source of truth for current behavior.
This roadmap includes future product direction and should not be used as proof
that experiment, parameter, workflow, device, provenance, AI automation, or
semantic-manifest behavior has already landed.

The existing dataset semantics proposal is the bridge between current dataset
behavior and the next product phase:

- `dataset-semantic-architecture-proposal.md` defines the proposed direction:
  explicit dataset semantics, durable manifests, resolved interpretation, and a
  shaped feature sequence that keeps the foundation separate from later scan
  and chart migration work. Treat it as proposed design guidance, not current
  implementation fact.

When implementation lands, update the current implementation notes and public
docs before treating proposal content as current behavior.

## Product Route

The current route is dataset-first, Python-led, and local-first.

Fricon should first become reliable for recording, organizing, and inspecting
scientific measurement datasets. Experiment, parameter, and device concepts
should build on that foundation instead of forcing the dataset layer to absorb
higher-level workflow meaning implicitly.

Initial experiment support should lean on Python scripts as the execution
entry point. The desktop UI should inspect, browse, and eventually assist those
workflows, but it should not become the primary experiment execution engine
before the Python-led model is clear.

Device management remains a later foundation. Near-term work may preserve space
for device identity and configuration, but should not build a broad driver or
hardware orchestration framework.

Workflow definitions are a later layer above individual experiments. They may
eventually coordinate repeated runs, scheduled calibration, optimization, and
benchmark tasks, but should build on run records, parameter snapshots, and
provenance first.

AI integration is a cross-cutting automation layer. It should help with
repetitive scientific data-management work, but actions that mutate data,
parameters, code, workflow definitions, or execution plans should require clear
user review and durable audit records.

## Product Pillars

- Workspace and dataset management
- Dataset semantics and durable interpretation
- Desktop dataset browsing, inspection, and charting
- Python scripting API for data recording and automation
- Python-led experiment and parameter workflows
- Reproducibility support for experiment code and environments
- Traceability across runs, datasets, parameters, code, and environments
- Workflow definitions and scheduled automation
- AI-assisted automation with explicit review and auditability
- Later device identity and configuration foundations

## Now

Current work should favor:

- stabilizing the local-first workspace and dataset experience
- landing the minimal durable dataset semantic model
- keeping dataset semantics separate from higher-level run, workflow, parameter,
  device, and AI concepts until those concepts have their own model boundaries
- keeping Python API and desktop UI behavior coherent as user-facing contracts
- preserving clear vertical slice boundaries across Rust, Python bindings, and
  the frontend
- keeping current implementation notes accurate when storage, IPC, or workspace
  behavior changes

Start with:

- `project-intent.md`
- `architecture-guidelines.md`
- `current-storage-notes.md`
- `maintenance-checklist.md`
- `pr-preflight-checklist.md`

## Next

Near-term direction should clarify and land the dataset semantic foundation,
then add progressive column metadata, explicit scan semantics, and desktop UI
consumption of resolved dataset interpretation instead of rediscovering chart
meaning from row heuristics.

Do this before building higher-level experiment, workflow, automation, or AI
features. Those later features need explicit run and provenance models; they
should not be smuggled into dataset naming conventions, incidental metadata, or
chart-specific assumptions.

Relevant notes:

- `dataset-semantic-architecture-proposal.md`
- `database-schema-changes.md`
- `release-and-versioning.md`

## Core Model Pressure

These future concepts have enough model impact that early dataset, run,
parameter, and workspace work should leave room for them. They are not all
near-term implementation scope, but ignoring them may cause avoidable schema,
API, or UI rewrites later.

- provenance graph across runs, datasets, parameters, code versions,
  environments, device configuration, workflow definitions, imports, exports,
  and notes
- parameter snapshots and parameter-set versioning instead of only mutable
  "current parameter" state
- unit, label, precision, and display-scale metadata for both dataset columns
  and parameters
- sample or specimen identity for workflows where the measured object matters
  as much as the device or parameter set
- dataset kind and lineage, including measured, imported, processed, and
  simulation datasets
- workflow definition, workflow run, and experiment run as distinct concepts
- local automation safety and approval boundaries for scheduler, optimizer, and
  AI-assisted actions
- event and audit log support for immutable records, corrections, manual
  overrides, failed automation, and AI-assisted changes

## After Dataset Semantics

After the dataset foundation is durable, product work should move toward:

- richer desktop inspection and charting workflows
- dataset read snippets that help users reopen or reproduce analysis from
  Python
- dataset preview, export, and plotting snippets for common Python workflows
- lightweight run notes, tags, favorites, and quality flags
- Python-led experiment run records
- parameter snapshots, diffs, and display for recorded runs
- a minimal model for connecting runs to datasets
- basic run provenance links between datasets, parameters, code, environment,
  and notes

This phase should avoid turning experiment support into a desktop-first
workflow engine too early. Python scripts should remain the first-class way to
run scientific measurement code.

## Future Product Concepts

These ideas are promising, but not current implementation commitments. Convert
them into focused design notes, issues, or ADRs before implementation if the
details affect storage, API contracts, or user workflows.

Parameter management may grow beyond static run metadata into:

- parameter history views
- plotting parameter values across runs or time
- parameter versioning for experiment configurations
- links between parameter versions, runs, and generated datasets

Experiment code management may support local reproducibility features such as:

- automatic history tracking for experiment code
- Git-backed storage, possibly using a bare repository managed inside the
  workspace
- automatic environment capture or setup using tools such as `uv` or `pixi`
- clear links between a run, the code version, the environment, parameters, and
  produced datasets

Dataset usability may include:

- generated Python read snippets for each dataset
- copyable examples for loading selected datasets by workspace-local ID or UID
- preview snippets for pandas, pyarrow, plotting, CSV export, and Parquet export
- snippets that match the public Python API instead of exposing internal
  storage layout
- saved table and chart views
- dataset aliases for important datasets
- compare views for selected datasets or runs

Scientific quality-of-life features may include:

- run notes for manual observations and experimental context
- sample or specimen records for experiments organized around a measured object,
  batch, preparation, condition, or source
- quick tags, favorites, and filters for datasets, runs, and parameter sets
- data quality flags such as good, suspect, failed, calibration, or test run
- calibration records linked to runs, parameters, and device configuration
- unit, label, precision, and display-scale metadata for parameters and dataset
  columns
- template experiments that copy parameter structure, code entry points, and
  output dataset conventions from previous work

Traceability and reproducibility may include:

- a provenance graph from workflow to run, datasets, parameters, code version,
  environment, device configuration, notes, imports, and exports
- immutable run records with later corrections recorded as appended events
- a workspace event timeline for dataset, run, parameter, import, export, and
  automation events
- parameter snapshots for each run and parameter diffs between runs
- input lineage for derived datasets, including measured, processed,
  simulation, and imported dataset categories
- sample or specimen provenance when runs and datasets are tied to physical
  measured objects
- import provenance such as source path, file hash, import time, and conversion
  options
- export provenance such as exported content, time, format, and destination
  summary
- checksums for dataset chunks, exported bundles, code snapshots, and
  environment lock files
- human-readable audit summaries for selected runs or workspaces

Workflow automation may include:

- workflow definitions above individual experiments
- reusable workflow templates
- workflow versioning and workflow run history
- scheduled or periodic workflows
- automatic parameter calibration and optimization
- periodic instrument or experiment benchmarks
- benchmark history and trend plots
- optimization objectives, constraints, chosen parameter changes, and rollback
  context
- human approval checkpoints for risky automation
- failed automation attempts with logs and partial outputs

AI-assisted automation may include:

- metadata cleanup suggestions
- generated read, plot, export, and report snippets
- parameter comparison and anomaly-explanation assistance
- tagging, note summarization, and draft report generation
- calibration or benchmark interpretation suggestions
- workflow drafts generated for user review before execution
- audit records for AI-assisted changes, including model/provider/version when
  practical and a task summary that avoids storing unnecessary sensitive prompt
  content

## Later

Longer-term product direction includes explicit concepts for:

- UI-assisted experiment execution
- richer parameter schemas and sweep definitions
- parameter history and version comparison workflows
- experiment code and environment history management
- run provenance and audit reporting
- workflow definitions and scheduled automation
- automatic calibration, optimization, and benchmark workflows
- AI-assisted repetitive-work automation with human approval boundaries
- device identity and configuration
- device integration points once real workflows justify them

Keep these ideas aligned with `project-intent.md`. Do not let future remote,
multi-user, or SaaS possibilities drive the current local-first architecture.

## Issue Planning Guidance

When generating GitHub issues from this roadmap:

- split work by product pillar and owning code area
- keep each issue small enough for one focused pull request
- mark whether the issue is backend, Python API, desktop UI, docs, or
  cross-boundary
- list likely touched files or modules when known
- list required validation commands from `pr-preflight-checklist.md`
- create an ADR task before implementation when a durable cross-cutting
  decision is still unresolved

## Explicit Non-Goals For Current Work

Do not treat these as active roadmap items unless the project direction changes
explicitly:

- multi-user collaboration
- hosted SaaS operation
- account, team, role, or permission systems
- distributed database semantics
- web-first deployment as the primary experience

## Decision Pressure

Create or update an ADR when a decision is durable, cross-cutting, and likely to
be rediscovered or relitigated later. Good ADR candidates include:

- workspace and storage format compatibility
- dataset semantic model commitments
- IPC or gRPC compatibility policy
- Python API contract decisions
- desktop/runtime architecture decisions
- experiment, parameter, or device model foundations
- experiment code history and environment management foundations
- run provenance and immutability foundations
- workflow definition and scheduler foundations
- AI action, approval, and auditability foundations

Use `dev-docs/adr/README.md` for ADR rules and format.
