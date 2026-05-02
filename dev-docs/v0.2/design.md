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
replace a simple LabRAD Grapher/Data Vault style logger for new measurements
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

- run exploratory measurements from Python with little boilerplate
- keep one coherent local data library instead of many data/code folders
- record sample and cooldown/session context beside measurements
- let users set active sample/session context from a notebook prelude or UI
  without making sample setup a hard requirement for quick measurements
- inspect live and historical datasets through Fricon Desktop and Python
- preserve parameter, code, environment, favorite, and lifecycle context for
  each measurement
- visualize sample/device parameters on a 2D map when the lab model needs it
- automate repeated calibration without silent parameter mutation
- leave a clean path to managed device communication after LabRAD is removed

v0.2 should not optimize for:

- hosted SaaS
- lab-wide administration
- full multi-user permission management
- a generic workflow DAG engine
- a broad hardware driver framework in the initial v0.2 implementation

## User-Visible Concept Budget

Fricon should not ask users to understand every internal provenance object.

User-visible concepts for the first v0.2 slice:

- Data Library: the local Fricon root and catalog
- Sample: the physical object, device, chip, wafer, batch, or specimen; useful
  when known, but not required before every quick measurement
- Sample Session: a cooldown, mounting, wiring, probing, or campaign context;
  can be selected as active context or attached after a run
- Measurement: the normal record for data-taking work
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

## Data Library Identity

A data library should have durable identity:

- generated `data_library_uuid`
- user-editable display name
- optional source computer label for human recognition
- schema/storage compatibility version

This identity should travel with exports, audit summaries, and portable
bundles. If a lab computer normally has one data library, the generated UUID and
display name are enough to answer where an exported measurement came from
without making users manage many roots.

## Naming Direction

v0.2 should use `Measurement` as the primary user-facing noun for a data-taking
attempt.

Useful reference patterns:

- LabRAD Data Vault/Grapher keeps the fast path close to datasets and live
  plotting. Fricon should keep that low-friction logging feel, but add explicit
  sample, parameter, code, and provenance records instead of relying on folder
  names.
- QCoDeS separates `Measurement`, `DataSet`, and `Experiment`/container-like
  grouping. Fricon should learn from that split: measurement is the acquisition
  attempt, dataset is the table-shaped output, and experiment can remain a
  broader scientific grouping if the product later needs it.
- Labber presents measurement setup and log-browser concepts. Fricon should
  use measurement for acquisition, but avoid making `Log` the primary data noun
  because software/audit logs are also first-class provenance.

Primary references:

- LabRAD Data Vault/Grapher quick start:
  https://sourceforge.net/p/labrad/wiki/QuickStartDataVaultAndGrapher/
- QCoDeS measurement and dataset example:
  https://microsoft.github.io/Qcodes/examples/DataSet/Performing-measurements-using-qcodes-parameters-and-dataset.html
- QCoDeS experiment container example:
  https://microsoft.github.io/Qcodes/examples/DataSet/The-Experiment-Container.html
- Keysight Labber product page:
  https://www.keysight.com/us/en/products/all-instrument-software/labber-software.html

Public naming policy:

- Use `Data Library`, `Sample`, `Sample Session`, `Measurement`, `Dataset`,
  `Artifact`, `Parameter Profile`, `Parameter Snapshot`, `Analysis`, and
  `Calibration` in user-facing docs and first APIs.
- Use `Experiment` informally for the scientific activity, or later for a
  broader campaign/template/grouping if an ADR proves that users need it.
- Avoid `ExperimentRun` as the first public API/storage noun. If implementation
  needs a durable lifecycle record, prefer `MeasurementRun` or keep the generic
  `ActivityRun` internal.
- Use `ActivityRun`, `DatasetWriteSession`, `ScriptRun`, `DeviceApplyPlan`, and
  similar names as internal or advanced troubleshooting concepts.
- Avoid `Station` as the main device/setup noun. Use `Device`, `Setup`,
  `DeviceSnapshot`, or `DesiredDeviceState` depending on what is being modeled.
- Avoid `Log` for measured data. Use `ExecutionLog`, `AuditLog`, or
  `EventLog` only when the record is actually a log.

## Favorites, Notes, Tags, And Lifecycle

Favorites, notes, tags, and lifecycle/status flags should attach to the level
where users naturally make decisions.

v0.2 should lead with favorites or pins as the main manual signal. This
matches how experimentalists often mark important data without maintaining a
taxonomy. Tags and notes should be easy to add from an optional drawer or
detail view, but they should not become required ceremony in the measurement
happy path.

Quality-like state should mostly come from lifecycle or system-managed facts:
incomplete, interrupted, calibration/test, invalidated, or superseded. Broad
manual labels such as good, suspect, or failed can be represented by optional
notes/tags or later customization if real users need them.

Default user-facing annotation levels:

- sample: favorite/pin, identity, preparation, layout, long-lived notes,
  aliases, optional tags
- sample session: favorite/pin, cooldown/setup context, wiring, drift notes,
  optional tags
- measurement: favorite/pin, intent, run notes, optional tags, lifecycle flags,
  interruption decisions
- analysis or calibration: conclusion, accepted/rejected proposals, audit note,
  lifecycle flags
- dataset: output-local exceptions, display hints, or dataset-specific
  lifecycle issues

Do not require users to annotate every dataset produced by a measurement.
Dataset-level notes or flags are useful when an output has a local issue or
meaning, but the measurement is the default annotation container for
data-taking work.

## Product Decision Checklist

Use this checklist before writing durable v0.2 storage, API, IPC, or UI
contracts.

Settled v0.2 product decisions:

- v0.2 replaces simple LabRAD Grapher/Data Vault style logging for new
  measurements. Full legacy import and browsing remain follow-up migration
  work.
- v0.2 may break the old model freely when compatibility would preserve the
  wrong workspace/dataset-first API, storage, or UI assumptions.
- The v0.2 user-facing slice should stay minimal: Data Library, Measurement,
  Dataset, and optional Sample/Sample Session. Parameter, analysis,
  calibration, actor, and provenance concepts may be recorded or reserved, but
  should not become peer navigation concepts before the measurement loop is
  ergonomic.
- v0.2 should not add first-class Project or Campaign as required grouping
  objects. Organize work through samples, sessions, measurements, favorites,
  search, saved views, and optional tags.
- Favorites or pins are the primary manual signal for important measurements.
  Tags and notes should be available but secondary.
- v0.2 status should emphasize system lifecycle flags such as
  incomplete, interrupted, calibration/test, invalidated, and superseded.
  Avoid making broad manual good/suspect/failed classification part of the
  happy path.
- Measurement-centered portable export is the default. Dataset-only export can
  remain a lower-level option; arbitrary library-subset export is later scope.
- Analysis provenance should be reserved in the model, but Analysis should not
  be a peer navigation concept in the first measurement-facing UI.
- Managed/declarative measurement remains a future path. v0.2 UX should stay
  ordinary Python measurement code.
- Shared lab computers should use a lightweight actor label or token for
  mutating actions. Do not add accounts, teams, roles, or permissions UI.
- Sample custom fields and 2D visualization are first-class design
  requirements, but implementation can follow the minimal measurement loop.
- Code/environment provenance should start as passive summaries, not managed
  code history or environment capture.

Decide at the product level before implementation:

- One data library is the normal user model. Saved views, optional tags,
  samples, and sessions organize the library without encouraging many
  long-lived roots.
- `Measurement` is the first public acquisition noun. `Experiment` is informal
  scientific language or a possible future grouping/template concept.
- Sample/session context is useful and visible, but optional. Quick
  measurements remain valid without it and can be corrected later.
- Measurement-scoped dataset writers are the common Python path. Lower-level
  dataset-only writes stay available for scratch tables, imports, tests, and
  transitional code.
- Portable exports start from a measurement by default and can be opened
  directly from Python or a read-only viewer without creating a local data
  library first.
- Favorites, optional notes/tags, and lifecycle flags usually live on samples,
  sessions, measurements, analysis, or calibration records; dataset
  annotations are for output-local exceptions.
- v0.2 may break workspace/dataset-first assumptions where they conflict with
  the new model.

Defer to ADR or technical design before implementation:

- data-library storage layout and pre-v0.2 compatibility or migration policy
- `MeasurementRun` table versus shared `ActivityRun` storage
- dataset facts, semantic manifests, projections, and append invariants
- measurement-scoped writer lifecycle, abort/finalize policy, and crash
  recovery semantics
- portable export bundle format, checksums, and direct-read API
- actor/auth token model for local and remote access
- minimal device adapter/capability boundary
- parameter snapshot/profile/proposal storage and calibration promotion rules
- managed measurement plan syntax, retry/resume behavior, and resource leases

Keep future-only until a narrower design proves the need:

- full multi-user administration, roles, and permission matrices
- hosted SaaS or distributed database operation
- full legacy LabRAD/Data Vault import or browsing
- broad hardware driver framework
- generic workflow DAG engine
- automatic notebook state capture
- full Git/environment management
- AI-driven mutating automation without explicit approval and audit records

## Domain Model

The clean v0.2 domain model is a data library of records plus explicit links,
not a strict containment tree:

```text
DataLibrary
  records:
    Sample
    SampleSession
    ActivityRun
      kind: measurement | analysis | import | simulation | calibration
    Artifact
      kind: dataset | analysis_result | report | log | attachment |
            parameter_proposal | device_snapshot
    ParameterProfile
    ParameterSnapshot
    CodeSnapshot
    Event/AuditRecord

  links:
    SampleSession -> Sample
    Measurement -> optional SampleSession
    ActivityRun -> consumes -> Artifact | ParameterSnapshot | ActivityRun
    ActivityRun -> produces -> Artifact
    CalibrationWorkflowRun -> coordinates -> Measurement + Analysis +
                                      ParameterProposal
```

More formally:

```text
ActivityRun consumes inputs and produces artifacts.

Sample/session context is an optional link on a measurement, not an owning
parent.

Calibration coordinates measurement, analysis, artifact, and parameter-proposal
records. It does not make those records children of the calibration record.

Measurement, Analysis, Import, Simulation, and Calibration are user-facing or
workflow-facing activity types.

Artifact is the general output or input concept. DatasetArtifact is the primary
table-shaped artifact for v0.2, while AnalysisResult, Report,
ParameterProposal, file attachments, logs, figures, code summaries, and future
device snapshots should fit the same provenance pattern.
```

The `ActivityRun` pattern should be an internal modeling tool, not the primary
word users see in the UI. Users should see Measurement, Analysis, Import,
Simulation, and Calibration as concrete work types. `Experiment` remains
available as informal scientific wording or as a future grouping if needed.

## Measurement, Experiment, And Simulation

Do not force measurement, experiment, and simulation into completely separate
foundations.

The clean distinction is:

- Measurement: one data-taking attempt against a physical system, hardware, or
  sample
- Experiment: an informal scientific activity label, or a future broader
  grouping/template/campaign above measurements
- Simulation: activity that computes synthetic or model-derived data
- ActivityRun: shared internal provenance pattern for all of them
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

A dataset is an artifact, not the whole measurement record.

Dataset artifacts should have stable identity, table facts, dataset-local
semantics, and direct Python access. They should not own sample identity,
measurement intent, parameter history, code state, or calibration decisions.

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

## Measurement-Scoped Writes

The common write path should share the measurement lifecycle.

In notebooks, the library handle should be created once in a prelude and reused
across cells. It should not be a context manager in normal examples; service
lifecycle and cleanup belong to the local Fricon service, CLI, or Fricon
Desktop, not to every notebook cell.

Prefer this shape for normal measurements:

```python
# Notebook prelude. Exact API unsettled.
lib = fricon.library()
lib.use_context(sample="sample-a", session="cooldown-2026-05")

with lib.measurement("rabi q3") as meas:
    rabi = meas.dataset("rabi")
    chevron = meas.dataset("chevron")

    for amp in amps:
        rabi.write(amp=amp, response=measure_rabi(amp))

    for freq, amp in points:
        chevron.write(freq=freq, amp=amp, signal=measure(freq, amp))
```

The active context may be set from a notebook prelude, CLI, Fricon Desktop, or
an explicit Python object. It should behave like a helpful default, not hidden
provenance. Each measurement should record the resolved sample/session IDs or
record that no sample context was selected.

Quick measurements should also be valid without sample context:

```python
lib = fricon.library()

with lib.measurement("quick resonator check") as meas:
    s21 = meas.dataset("s21")
    s21.write(freq=7.1e9, signal=measure())
```

Users should be able to attach or correct sample/session links later through an
auditable correction path.

Dataset handles opened from a measurement should finalize with the measurement
unless the user explicitly aborts or detaches them. This avoids nested writer
ceremony for the normal case while preserving explicit dataset lifecycle for
lower-level or streaming APIs.

Lower-level dataset creation remains useful:

```python
lib = fricon.library()

with lib.dataset("scratch") as ds:
    ds.write(x=1.0, y=2.0)
```

Lower-level datasets are unassigned artifacts until linked to a producer.

## Portable Measurement Export

Export should be measurement-centered by default.

The common workflow is:

```text
measurement in lab data library
  -> export portable measurement bundle
  -> open bundle directly on another computer from Python or a viewer
```

Users should not need to create a new local data library, import the bundle, or
understand Fricon storage internals before analyzing exported data.

A measurement export should include:

- export format version and export UUID
- source data library UUID, display name, and optional source computer label
- exported-at time, Fricon version, and actor summary when available
- original measurement/run IDs and stable artifact IDs
- measurement name, favorite/pin state, notes, optional tags, lifecycle flags,
  and correction summaries
- optional sample/session context and attach-later correction history
- produced dataset artifacts with facts, semantic manifests, and projections
- non-table artifacts such as reports, figures, logs, attachments, code
  summaries, and future device snapshots when selected
- parameter snapshot or legacy parameter JSON used by the run when available
- code and environment summary when available
- provenance links needed to explain inputs, outputs, analysis, and calibration
- checksums for payload files and manifest records
- generated Python read snippets for the portable API

Portable read APIs should be read-only and direct:

```python
bundle = fricon.open_export("rabi-q3.fricon-export")
meas = bundle.measurement("rabi q3")
rabi = meas.dataset("rabi").to_pyarrow()
```

Fricon Desktop should also have a read-only export viewer mode. The viewer
should open exported bundles directly, show measurement context and produced
artifacts, and offer read snippets without requiring import into the user's own
data library.

Importing an export into another data library may be useful later, but it is a
separate workflow from reading or viewing the exported measurement.

## Sample And Session Model

The sample model should replace the old habit of using data-vault directories
as sample boundaries.

Sample/session context should be easy to set but not mandatory for quick
measurements. Many existing notebooks start with a small prelude such as
`data_dir = ...`; Fricon should support an equivalent active sample/session
prelude and a desktop active-context selector.

A measurement may start with:

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

- measurement facts should link to immutable snapshots, not mutable JSON files
- run-local overrides should be recorded on the run
- analysis and calibration should produce proposals before profile updates
- automatic calibration should not silently mutate important profiles during
  measurement
- parameter display, diff, and search should be first-class UI concerns after
  the minimal measurement and dataset foundation exists

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

## Managed Measurement Model

Simple measurements should remain ordinary Python.

Managed measurement plans should be a future optional integration path for repeated,
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

Imperative measurement code remains valid. It just gets less automatic help with
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

The initial v0.2 implementation can record only summaries, placeholders, or a very
thin adapter contract. The important decisions are:

- device state belongs to managed execution provenance, not dataset metadata
- Fricon should have a typed place for future device capabilities and readbacks
- dummy devices and dry runs should use the same boundary as real adapters
- LabRAD compatibility, if needed during migration, should be an adapter behind
  the boundary rather than the conceptual model

## Analysis And Calibration

Analysis is a consumer and producer.

It may consume measurement artifacts, usually datasets, and produce:

- derived datasets
- scalar or structured results
- reports
- quality decisions
- parameter proposals

Calibration is a workflow over measurement and analysis records:

```text
CalibrationWorkflowRun
  -> measurement or managed measurement
  -> measured dataset
  -> analysis result
  -> parameter proposal
  -> validation
  -> approval or policy gate
  -> parameter profile update
```

This keeps measurement outputs clean while still letting the UI show calibration
history inside the broader measurement/sample context.

## Distribution And Runtime

v0.2 should expose three public surfaces:

- Fricon Desktop
- `fricon` CLI
- Python SDK

They should operate against one local data library and one local service:

```text
Fricon Desktop / CLI / Python SDK
  -> local Fricon service
  -> Fricon data library
```

Fricon Desktop may be implemented as a web application packaged with Tauri.
For v0.2, Fricon Desktop is the shipped GUI and runs in local mode. Remote
mode, browser-served UI, and PWA-style access are future targets.

The desktop app may launch or supervise the local service, but the service is
the authoritative data backend. Core measurement, dataset, sample/session,
note/tag, and live-chart features should use the service API rather than Tauri
commands so future desktop remote mode or browser clients can reuse the same UI
and compatibility boundary.

Feature code should avoid unnecessary Tauri-only assumptions. Tauri-specific
capabilities should stay at the shell boundary: file pickers, offline bundle
file association, updater, diagnostics, window management, and local service
launch.

Installation and update are product-visible v0.2 concerns. Users should get one
coherent Fricon release shape for Fricon Desktop, CLI, Python SDK, and local
service, with clear compatibility checks before any client mutates a data
library.

The exact protocol can remain a technical ADR, but v0.2 should not leave
client/server mismatch behavior undefined. Because Python measurement code may
be pinned by virtual environments or lockfiles, v0.2 should make the core v0.x
Python SDK measurement-write and dataset-read path compatible with later v0.x
local services. Newer features should use explicit capability negotiation.

## Future Remote And Auth Boundary

Remote access should be planned early, but v0.2 should not ship remote mode or
become a multi-user system.

The recommended boundary is:

- local loopback access can use implicit local trust or a generated local token
- mutating operations record an actor
- the first authorization model is single-owner
- future remote access requires token or pairing
- roles, teams, and permission matrices remain future scope

Auth is hard to retrofit when APIs and audit logs have no actor field. Add the
actor boundary early; defer full multi-user product complexity.

Do not support shared-folder multi-machine access to the same database-backed
data library. Future remote viewing should connect to the Fricon service that
owns the library.

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

The initial v0.2 engineering slice should prove the new model end to end:

1. Create or open one data library.
2. Create or select a sample/session when known, or explicitly run without one.
3. Set active sample/session context from Fricon Desktop or Python prelude when
   appropriate.
4. Start an interactive measurement from Python.
5. Write one or more dataset artifacts through measurement-scoped handles.
6. Browse the measurement and datasets in Fricon Desktop.
7. Reopen a dataset from Python by stable ID.
8. Attach or correct sample/session context after the run when needed.
9. Record actor, passive code summary, favorite/pin state, run note, lifecycle
   flags, and sample/session links.

This slice intentionally breaks old workspace/dataset assumptions where they
conflict with the v0.2 model.

## ADRs Needed Before Durable Implementation

Create ADRs before committing durable storage, API, or IPC contracts for:

- data library versus workspace public model
- supported distribution surfaces and install/update policy
- sample and sample-session identity
- active sample/session context and attach-later correction policy
- general Artifact versus DatasetArtifact boundary
- dataset artifact and provenance model
- public naming policy for Measurement versus Experiment
- measurement-scoped dataset writer lifecycle
- minimal device adapter/capability boundary for future LabRAD replacement
- actor/auth boundary for local access and future remote access
- client/server protocol compatibility, version negotiation, and core v0.x
  Python SDK compatibility policy
- optional managed measurement desired-device-state boundary
- storage compatibility and migration policy for pre-v0.2 workspaces
- Fricon Desktop shell boundary, local service ownership, and future
  remote/browser UI access

## Design Constraints

- Keep the Python measurement path simple.
- Keep datasets directly openable from Python and Fricon Desktop.
- Keep sample/session context easy to set as an active default, but do not block
  quick measurements when the context is unknown.
- Keep dataset semantics dataset-local.
- Keep sample/session/run/parameter/provenance out of dataset names.
- Keep favorites, optional notes/tags, and lifecycle flags mostly on sample,
  session, measurement, analysis, and calibration records.
- Keep advanced execution concepts optional until users need retry, resume,
  dry-run, or calibration automation.
- Keep v0.2 local-only and single-owner in the product, while leaving actor,
  service API, and future remote-access hooks in the architecture.
