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
that measurement, parameter, workflow, device, provenance, AI automation, or
semantic-manifest behavior has already landed.

The proposed v0.2 reset is captured under `dev-docs/v0.2/`:

- `v0.2/README.md` is the canonical entry point for v0.2 planning.
- `v0.2/design.md` is the canonical v0.2 design synthesis. It defines the
  clean reset model and explicitly separates the first shipped v0.2 slice from
  reserved future concepts. The first slice is one local data library, optional
  sample/session context, explicit measurements, table-shaped dataset
  artifacts, scan schema for plotted data, notes/events, lifecycle flags,
  honest code provenance, portable exports, backup/restore, and guided local
  diagnostics. Analysis, simulation, calibration, managed code snapshots,
  managed execution, and device communication are reserved future layers unless
  an ADR narrows a minimal placeholder.
- `v0.2/product-direction.md` repositions Fricon as a local lab data library
  and automation foundation centered on samples, sessions, measurements,
  dataset artifacts, parameter history, code provenance summaries, and calibration
  history.
- `v0.2/technical-direction.md` defines the proposed distribution surfaces,
  data-library service model, Fricon Desktop shell direction, local-only v0.2
  scope, future remote/auth boundary, and rewrite strategy.

Archived pre-adoption proposal notes remain historical background behind the
v0.2 reset. Use them for rationale and edge cases only; do not treat them as
active scope, sequencing, or naming guidance when they conflict with
`v0.2/design.md`, `v0.2/product-direction.md`, or
`v0.2/technical-direction.md`:

- `v0.2/archive/measurement-system-foundation-redesign.md` preserves an older
  redesign proposal for dataset artifacts, run-like producer/consumer
  provenance, and future calibration foundations.
- `dataset-semantic-architecture-proposal.md` defines the detailed dataset
  semantic direction: explicit dataset semantics, durable manifests, resolved
  interpretation, and a shaped feature sequence that keeps the foundation
  separate from later scan and chart migration work. Treat it as proposed
  design guidance, not current implementation fact, and reconcile it with the
  broader redesign before implementation.
- `v0.2/archive/parameter-management-design.md` preserves an older long-term
  parameter registry proposal. Treat it as background input for a future
  rewritten parameter design, not as active v0.2 implementation guidance.
- `v0.2/future-concepts.md` preserves lightweight notes for concepts that are
  not yet ready for focused design proposals. Some entries, such as measurement
  records and sample identity, are now partially promoted into v0.2; treat the
  remaining runner, workflow automation, calibration automation, device apply,
  and AI automation parts as future.

When implementation lands, update the current implementation notes and public
docs before treating proposal content as current behavior.

## Product Route

The product route is the v0.2 measurement-library reset: Python-led,
local-first, and measurement-centered.

The current implementation is still workspace/dataset-first. Treat that as
implementation baseline only. New product and architecture work should point
toward the v0.2 reset unless it is explicitly maintaining current behavior.

v0.2 should replace the old simple logger for new measurement work by recording
measurements and produced datasets together. Dataset semantics are still
foundational, but they should be developed as part of the measurement slice:
explicit scan schema, column metadata, and chart interpretation should make
live and historical measurement browsing reliable.

Datasets remain first-class artifacts, while samples, sample sessions,
measurements, optional parameter snapshots, code provenance summaries,
favorites, optional notes/tags, lifecycle flags, exports, and audit events get
explicit boundaries. `Experiment` can remain an informal scientific term or a
future grouping/template concept, but v0.2 should prefer `Measurement` as the
user-facing acquisition record.

v0.2 is local-only. Fricon Desktop, the CLI, and the Python SDK should talk to
one local service that owns one local data library. Remote mode,
browser-served UI, and PWA-like access are future product surfaces that should
shape the service/API boundary, but they are not shipped v0.2 requirements.

The ordinary v0.2 model is one primary local data library per machine. Windows
is the first-class lab-computer target; macOS remains supported for development
and normal local use.

Multi-computer labs should share measurement code and reusable setup assets
without sharing the active data library. v0.2 should record honest code
provenance levels for measurements. Later slices can add experimenter-friendly
"set up this computer" and "update approved lab code" workflows on top of
Gitea, GitLab, GitHub, bare Git mirrors, read-only network mirrors, or package
caches without requiring Fricon itself to become a central server.

Initial measurement support should lean on Python scripts as the execution
entry point. Fricon Desktop should inspect, browse, and eventually assist those
workflows, but it should not become the primary measurement execution engine
before the Python-led model is clear.

Device management remains a later foundation. Near-term work may preserve space
for device identity and configuration, but should not build a broad driver or
hardware orchestration framework.

Workflow definitions are a later layer above individual measurements. They may
eventually coordinate repeated runs, scheduled calibration, optimization, and
benchmark tasks, but should build on run records, parameter snapshots, and
provenance first.

AI integration is a cross-cutting automation layer. It should help with
repetitive scientific data-management work, but actions that mutate data,
parameters, code, workflow definitions, or execution plans should require clear
user review and durable audit records.

## Product Pillars

- Data-library, optional sample/session context, and artifact management
- Dataset semantics and durable interpretation
- Desktop dataset browsing, inspection, and charting
- Python scripting API for data recording and automation
- Python-led measurement and parameter workflows
- Reproducibility support for measurement code and environments
- Repeatable lab-computer setup and measurement-code source provenance
- Traceability across runs, datasets, parameters, code, and environments
- Workflow definitions and scheduled automation
- AI-assisted automation with explicit review and auditability
- Later device identity and configuration foundations

## v0.2 Replacement Target

v0.2 should be able to replace a simple LabRAD Grapher/Data Vault style
measurement logger for new measurement work.

Replacement means a researcher can stop using the old simple logger for new
measurements and use Fricon instead to:

- record table-shaped measurement datasets from Python scripts or notebooks
- run measurement scripts headlessly without opening Fricon Desktop first
- select or create a lightweight sample and sample-session context when the
  measured object matters, without blocking quick measurements when context is
  not yet known
- see newly produced datasets in Fricon Desktop without manual file handling
- inspect live, recent, and historical measurements through table, line/scatter,
  and basic 2D views
- provide explicit scan schema for datasets intended for live or historical
  plotting, including setpoint/independent, measured/dependent, fixed/config,
  monitor/readback roles, and enough axis structure for slicing and display
- keep live plotting, previews, export preparation, and future analysis hooks
  from slowing or failing acquisition writes
- use measurement display identity based on name/title, start time, and
  sample/session label, with stable IDs as secondary technical references
- record optional setup/method labels and basic clock/timing metadata without
  building device management
- record an honest code provenance level for each measurement. Non-managed
  user-run Python may be `unmanaged` or user-supplied summary; future managed
  runs can require a resolved immutable code snapshot before execution.
- install and launch Fricon Desktop, the CLI, Python SDK, and local service as a
  coherent Fricon release rather than assembling mismatched components manually
- use the CLI primarily for setup, diagnostics, service control, and developer
  workflows rather than as the normal experimenter workflow surface
- install the local service as a bundled sidecar instead of requiring a
  separate server install for ordinary local use
- ask for the data-library location on first run and remember it
- use basic backup/restore, trash/recover, and explicit migration checkpoints as
  user-visible safety paths
- update Fricon with clear compatibility checks for the data library and
  local-service protocol
- stage service/Desktop updates and apply them only when active measurements,
  open writers, imports, exports, and migrations are idle
- move public Fricon clients toward one HTTP/WebSocket service API with binary
  dataset payload endpoints instead of separate UI and Python gRPC contracts
- protect recorded data across v0.x with explicit migrations and
  fail-before-write diagnostics, while allowing SDK, CLI, UI, and protocol APIs
  to break when necessary
- use explicit dataset semantics for column metadata, scan axes, chart defaults,
  and live-view interpretation instead of relying on row-order heuristics
- create an explicit but low-ceremony measurement record that groups produced
  datasets
- highlight new live measurements in Desktop without stealing focus or opening
  windows automatically
- reserve a general artifact model so reports, logs, figures, attachments, and
  future device snapshots do not have to masquerade as datasets
- store measurement-level names, favorite or pin state, optional notes/tags,
  lifecycle flags, and legacy JSON metadata on the run
- keep dataset metadata focused on output-local semantics and per-output
  exceptions
- reopen produced datasets from Python using generated read snippets or stable
  data-library identifiers
- preserve interrupted partial data; reruns create new linked measurements by
  default, while continuation requires explicit intent and compatibility checks
- export measurements as read-only analysis packages with practical common
  tabular files, a simple manifest/index preview, checksums, units, notes/events,
  loader snippets, and direct Python/Desktop offline-viewer access
- include light measurement attachments such as small files, images, or logs
  without building a full artifact management UI

v0.2 replacement does not require:

- importing or fully browsing legacy LabRAD/Data Vault history
- a LabRAD Data Vault/Grapher compatibility layer for old scripts
- multi-user LabRAD server semantics or hosted collaboration
- desktop-first measurement execution
- broad ordinary-user CLI workflows
- a generic managed runner, queue, resource lease system, or workflow engine
- a full parameter registry or device driver framework
- post-hoc scan guessing as the primary path for plotted measurement data
- Fricon-managed device communication
- automatic calibration workflows
- managed analysis records or analysis-run UI
- driver marketplace or shared-driver ecosystem
- regulated-lab compliance UX as a v0.2 product goal
- automatic Git, `uv`, or `pixi` environment management
- a full measurement-code sync system, Git GUI, hosted code service, or
  central Fricon server for distributing code
- running active measurement code directly from a shared network folder as the
  primary workflow
- a long-term third-party client protocol stability promise
- a complex auto-update system before the first replacement workflow is proven
- remote mode, browser-served UI, or PWA distribution as shipped v0.2 features
- remote annotations or remote acquisition writes
- Labber-like visual sweep builder as a product goal
- opening the same database-backed data library directly from multiple
  computers through a shared folder

## Primary User Mental Model

v0.2 should optimize the user-facing product around interactive measurement
records:

```text
interactive measurement -> produced datasets -> inspection, export, and analysis
```

For measurement work, examples and desktop navigation may become run-first once
the run API exists. The dataset remains a first-class artifact with stable
identity, dataset-local semantics, and direct Python access.

Do not model datasets as owned exclusively by measurements. A dataset may be:

- produced by a measurement
- produced by a future analysis, import, simulation, or calibration activity
- consumed by later analysis or calibration work
- temporarily unassigned when created through lower-level dataset-only APIs

This keeps the v0.2 LabRAD-style workflow simple while leaving room for later
provenance graph where activity records consume and produce datasets, artifacts,
analysis results, and parameter proposals.

## v0.3/v0.4 Candidate Direction

After v0.2 proves the core measurement loop, v0.3 and v0.4 should add product
help in small slices rather than reopening the foundation each time.

The leading v0.3 candidate is read-only LAN viewing from another computer,
because it preserves the single-owner data-library model while covering an
existing LabRAD Grapher usage pattern.

Likely v0.3 candidates:

- read-only LAN viewing from another computer through the service API
- richer sample fields and 2D sample maps
- attach/correct context UX polish
- comparison and saved-view workflows
- lightweight measurement templates for repeated names, scan schemas, and
  display defaults
- portable export viewer polish
- passive setup/device snapshots with software, firmware, driver, method, and
  calibration-state summaries
- compare-what-changed views across sample, setup/method labels, parameters,
  code, operator, calibration state, and produced data
- rerun-from-artifact workflow seeded by a previous measurement's scan schema,
  labels, code/method summary, and display defaults
- smoother installer/update polish after the v0.2 release shape is proven
- offline/silent installer, side-by-side versions, and rollback support for
  locked-down Windows lab PCs
- code provenance/environment summary improvements
- guided "set up this computer" onboarding that installs or checks Fricon,
  selects a local data library, connects to the lab measurement-code source,
  prepares the Python environment, selects a setup profile, and runs diagnostics
- measurement-code source manager for approved releases/tags, simple update
  flows, local-change warnings, and change summaries aimed at users who do not
  want to operate Git directly
- support for Gitea/Git remotes, bare mirrors, read-only network mirrors, or
  package caches as code/install distribution aids, while keeping execution in
  a local checkout and data in the local data library
- better lifecycle/favorite filtering
- richer search by setup, operator, parameter, method/config, instrument label,
  and calibration state

Likely v0.4 candidates:

- parameter snapshot/proposal UI
- analysis provenance UI
- calibration workflow history
- managed measurement plans for repeated scans
- managed code snapshots, execution worktrees, and script-run provenance for
  runner-owned execution
- early device adapter and readback boundaries

These are candidate directions, not release commitments. Convert them into
focused issues or ADRs after the v0.2 replacement slice is usable.

## Now

Current work should favor the v0.2 reset:

- replacing the public workspace mental model with one local data library
- landing minimal measurement records that group produced datasets
- landing explicit scan schema and dataset semantics needed for reliable live
  and historical plotting
- keeping table-shaped dataset artifacts directly reopenable from Python
- adding optional sample/session context with attach-later correction
- adding honest code provenance levels without claiming verified code history
  for non-managed Python
- adding measurement-centered export, backup/restore, trash/recover, and guided
  local diagnostics
- keeping Python API and desktop UI behavior coherent as user-facing contracts
- preserving clear vertical slice boundaries across Rust, Python bindings, and
  the frontend
- keeping current implementation notes accurate when storage, IPC, or
  data-library behavior changes

Start with:

- `project-intent.md`
- `architecture-guidelines.md`
- `current-storage-notes.md`
- `maintenance-checklist.md`
- `pr-preflight-checklist.md`

## Next

Near-term direction should make the v0.2 replacement slice usable end to end,
then add product assistance in small steps.

Do not reintroduce the older sequencing where dataset semantics must be
finished before any measurement model can land. The first slice needs both:
minimal measurement records and enough explicit dataset semantics for plotted
measurement data.

After that slice works, add small v0.3/v0.4 improvements such as read-only LAN
viewing, sample-map polish, compare views, code-source setup/update UX,
parameter snapshot UI, analysis provenance, calibration history, and managed
measurement plans.

Relevant notes:

- `dataset-semantic-architecture-proposal.md`
- `database-schema-changes.md`
- `release-and-versioning.md`

## Core Model Pressure

These future concepts have enough model impact that early dataset, run,
parameter, and data-library work should leave room for them. They are not all
near-term implementation scope, but ignoring them may cause avoidable schema,
API, or UI rewrites later.

- provenance graph across runs, datasets, parameters, code versions,
  environments, device configuration, workflow definitions, imports, exports,
  and notes
- parameter snapshots, refs, drafts, table sections, tree sections, and
  parameter-set versioning instead of only mutable "current parameter" state
- unit, label, precision, and display-scale metadata for both dataset columns
  and parameters
- sample or specimen identity for workflows where the measured object matters
  as much as the device or parameter set
- dataset kind and lineage, including measured, imported, processed, and
  simulation datasets
- a shared producer/consumer provenance pattern for future measurement,
  analysis, import, simulation, calibration, and workflow runs
- workflow definition, workflow run, and measurement/activity records as
  distinct concepts
- local automation safety and approval boundaries for scheduler, optimizer, and
  AI-assisted actions
- event and audit log support for immutable records, corrections, manual
  overrides, failed automation, and AI-assisted changes

## v0.3+ Product Direction

After the v0.2 replacement slice is durable, product work should move toward:

- richer desktop inspection and charting workflows
- dataset read snippets that help users reopen or reproduce analysis from
  Python
- dataset preview, export, and plotting snippets for common Python workflows
- richer measurement filtering, comparison, favorite/pin, optional notes/tags,
  lifecycle flags, and legacy metadata workflows
- parameter snapshots, diffs, and display for recorded runs
- analysis provenance UI
- calibration workflow history
- measurement-code source setup/update UX
- managed code snapshots and script-run provenance after the runner boundary is
  explicitly designed

Relevant future design note:

- `v0.2/archive/experiment-run-and-runner-design.md`
- `v0.2/archive/parameter-management-design.md`

This phase should avoid turning measurement support into a desktop-first
workflow engine too early. Python scripts should remain the first-class way to
run scientific measurement code.

## Future Product Concepts

These ideas are promising, but not current implementation commitments. Convert
them into focused design notes, issues, or ADRs before implementation if the
details affect storage, API contracts, or user workflows.

Use `v0.2/future-concepts.md` for lightweight notes that should be preserved
but are too early for detailed design or issue planning.

Parameter management may grow beyond static run metadata into:

- immutable parameter snapshots that contain one optional tree section and
  multiple table sections
- mutable refs or profiles that resolve to immutable snapshots before a run
- parameter history views
- plotting parameter values across runs or time
- parameter versioning for measurement and numerical simulation configurations
- structured tree and table diffs with selected apply-to-draft workflows
- analysis-driven parameter update proposals for calibration workflows
- links between parameter versions, runs, and generated datasets

Measurement code management may support local reproducibility features such as:

- automatic history tracking for measurement code
- shared measurement-code sources or lab code packages used across acquisition
  computers
- Git-backed storage, approved release tags, a self-hosted Git service such as
  Gitea, or a read-only network mirror that Fricon can inspect without becoming
  the lab's Git host
- managed code snapshots resolved from a code source, cached in a local bare
  mirror where useful, and expanded into temporary execution worktrees for
  ScriptRuns
- automatic environment capture or setup using tools such as `uv` or `pixi`
- setup profiles, scan-schema helpers, measurement templates, plot presets,
  calibration workflow definitions, driver/helper modules, and environment
  lock files that can be reused across lab computers
- experimenter-facing actions such as install approved code, update to latest
  approved release, show what changed, check environment, open scripts folder,
  and export local changes for review
- clear links between a run, the code version, the environment, parameters, and
  produced datasets

Measurement-code management should not make the active data library shared.
Each acquisition computer should keep its own local data library, local code
checkout, local environment, machine-specific setup overrides, tokens, secrets,
and temporary notebook state.

Dataset usability may include:

- generated Python read snippets for each dataset
- copyable examples for loading selected datasets by data-library-local ID or
  UID
- measurement-centered portable export bundles for offline analysis on another
  computer
- direct Python APIs for opening exported bundles without creating or importing
  into a local data library
- a read-only desktop export viewer for exported measurement bundles
- preview snippets for pandas, pyarrow, plotting, CSV export, and Parquet export
- snippets that match the public Python API instead of exposing internal
  storage layout
- saved table and chart views
- dataset aliases for important datasets
- compare views for selected datasets or runs

Scientific quality-of-life features may include:

- run notes for manual observations and experimental context
- sample or specimen records for measurements organized around a measured object,
  batch, preparation, condition, or source
- favorites, saved filters, and optional tags for datasets, runs, and
  parameter sets
- lifecycle flags such as incomplete, interrupted, calibration, test,
  invalidated, or superseded
- calibration records linked to runs, parameters, and device configuration
- unit, label, precision, and display-scale metadata for parameters and dataset
  columns
- template measurements that copy parameter structure, code entry points, and
  output dataset conventions from previous work

Traceability and reproducibility may include:

- a provenance graph from workflow to run, datasets, parameters, code version,
  environment, device configuration, notes, imports, and exports
- immutable run records with later corrections recorded as appended events
- a data-library event timeline for dataset, run, parameter, import, export,
  and automation events
- parameter snapshots for each run and parameter diffs between runs
- input lineage for derived datasets, including measured, processed,
  simulation, and imported dataset categories
- sample or specimen provenance when runs and datasets are tied to physical
  measured objects
- import provenance such as source path, file hash, import time, and conversion
  options
- export provenance such as exported content, time, format, and destination
  summary
- source data library UUID, display name, and optional source computer label in
  exported measurement bundles
- measurement-code source label, revision or tag, script entry point, and
  environment summary in run provenance and exported measurement bundles when
  available
- checksums for dataset chunks, exported bundles, code snapshots, and
  environment lock files
- human-readable audit summaries for selected runs or data libraries

Workflow automation may include:

- workflow definitions above individual measurements
- reusable workflow templates
- workflow versioning and workflow run history
- scheduled or periodic workflows
- automatic parameter calibration and optimization
- periodic instrument or measurement benchmarks
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

- UI-assisted measurement execution
- richer parameter schemas and sweep definitions
- parameter history and version comparison workflows
- measurement code and environment history management
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
- a central Fricon server required only to distribute measurement code
- a full Git forge, Git teaching tool, or broad source-control client for
  ordinary experimenter workflows
- shared editable network folders as the primary way to run measurement code
- direct multi-machine access to the same database-backed data library through
  a shared folder

## Decision Pressure

Create or update an ADR when a decision is durable, cross-cutting, and likely to
be rediscovered or relitigated later. Good ADR candidates include:

- workspace and storage format compatibility
- dataset semantic model commitments
- distribution surfaces, installer/update policy, and release compatibility
- service API transport and compatibility policy, including the HTTP/WebSocket
  direction, binary dataset payload endpoints, and fail-before-write behavior
  for locked lab environments when APIs or protocols break
- Python API contract decisions
- desktop/runtime architecture decisions
- measurement, parameter, or device model foundations
- measurement code history and environment management foundations
- shared measurement-code source or lab code package model, including
  new-computer setup and the boundary between Fricon-managed actions and
  ordinary Git/environment tools
- run provenance and immutability foundations
- workflow definition and scheduler foundations
- AI action, approval, and auditability foundations

Use `dev-docs/adr/README.md` for ADR rules and format.
