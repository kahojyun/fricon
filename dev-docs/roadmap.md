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
  clean reset model: one data library; samples and sessions; measurement,
  analysis, simulation, import, and calibration activity records; dataset
  artifacts as the first concrete artifact type; reserved general artifacts;
  parameter snapshots and proposals; code summaries; actor/audit boundaries;
  optional active sample/session context; the public naming policy that prefers
  `Measurement` over `ExperimentRun` for data-taking records; and an optional
  managed measurement path toward Fricon-managed device communication.
- `v0.2/product-direction.md` repositions Fricon as a local lab data library
  and automation foundation centered on samples, sessions, measurements,
  dataset artifacts, parameter history, code summaries, and calibration
  history.
- `v0.2/technical-direction.md` defines the proposed distribution surfaces,
  data-library service model, Fricon Desktop shell direction, local-only v0.2
  scope, future remote/auth boundary, and rewrite strategy.

The pre-adoption measurement-system redesign note remains supporting background
behind the v0.2 reset:

- `v0.2/measurement-system-foundation-redesign.md` defines the proposed breaking
  redesign direction for dataset artifacts, run-like producer/consumer
  provenance, the v0.2 measurement-first mental model, and future calibration
  foundations. Treat historical `ExperimentRun` wording there as supporting
  proposal terminology unless reconciled by ADR. Treat it as proposed design
  guidance, not current implementation fact.
- `dataset-semantic-architecture-proposal.md` defines the detailed dataset
  semantic direction: explicit dataset semantics, durable manifests, resolved
  interpretation, and a shaped feature sequence that keeps the foundation
  separate from later scan and chart migration work. Treat it as proposed
  design guidance, not current implementation fact, and reconcile it with the
  broader redesign before implementation.
- `v0.2/parameter-management-design.md` defines proposed long-term parameter
  registry direction after dataset semantics and minimal run records exist.
  Treat it as future design guidance, not current implementation fact.
- `v0.2/future-concepts.md` preserves lightweight notes for measurement run,
  workflow automation, calibration automation, device apply, and AI automation
  concepts that are not yet ready for focused design proposals.

When implementation lands, update the current implementation notes and public
docs before treating proposal content as current behavior.

## Product Route

The current route is dataset-first, Python-led, and local-first.

Fricon should first become reliable for recording, organizing, and inspecting
scientific measurement datasets. Measurement, parameter, and device concepts
should build on that foundation instead of forcing the dataset layer to absorb
higher-level workflow meaning implicitly.

The proposed v0.2 reset keeps the Python-led and local-first constraints but
expands the product model from a dataset catalog into a measurement record
system. In that model, datasets remain first-class artifacts, while samples,
sample sessions, measurements, analysis, calibration, parameter snapshots, code
summaries, favorites, optional notes/tags, and lifecycle flags get explicit
boundaries. `Experiment`
can remain an informal scientific term or a future grouping/template concept,
but v0.2 should prefer `Measurement` as the user-facing acquisition record.

v0.2 is local-only. Fricon Desktop, the CLI, and the Python SDK should talk to
one local service that owns one local data library. Remote mode,
browser-served UI, and PWA-like access are future product surfaces that should
shape the service/API boundary, but they are not shipped v0.2 requirements.

The ordinary v0.2 model is one primary local data library per machine. Windows
is the first-class lab-computer target; macOS remains supported for development
and normal local use.

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
  plotting, including independent/dependent roles and enough axis structure for
  slicing and display
- use measurement display identity based on name/title, start time, and
  sample/session label, with stable IDs as secondary technical references
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
- export measurements as read-only portable bundles with practical common
  tabular files, a simple manifest/index preview, and direct Python/Desktop
  offline-viewer access
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
- automatic Git, `uv`, or `pixi` environment management
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
- smoother installer/update polish after the v0.2 release shape is proven
- passive code/environment summary improvements
- better lifecycle/favorite filtering

Likely v0.4 candidates:

- parameter snapshot/proposal UI
- analysis provenance UI
- calibration workflow history
- managed measurement plans for repeated scans
- early device adapter and readback boundaries

These are candidate directions, not release commitments. Convert them into
focused issues or ADRs after the v0.2 replacement slice is usable.

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

Do this before building higher-level measurement, workflow, automation, or AI
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
- workflow definition, workflow run, and measurement run as distinct concepts
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
- minimal interactive measurement records that make new simple measurement
  workflows no longer depend on LabRAD Grapher/Data Vault style logging
- run-level favorite or pin state, optional notes/tags, lifecycle flags, and
  legacy JSON metadata for migration from existing scripts
- parameter snapshots, diffs, and display for recorded runs
- a minimal model for connecting runs to datasets
- basic run provenance links between datasets, parameters, code, environment,
  and notes

Relevant future design note:

- `v0.2/experiment-run-and-runner-design.md`
- `v0.2/parameter-management-design.md`

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
- Git-backed storage, possibly using a bare repository managed inside the
  workspace
- automatic environment capture or setup using tools such as `uv` or `pixi`
- clear links between a run, the code version, the environment, parameters, and
  produced datasets

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
- run provenance and immutability foundations
- workflow definition and scheduler foundations
- AI action, approval, and auditability foundations

Use `dev-docs/adr/README.md` for ADR rules and format.
