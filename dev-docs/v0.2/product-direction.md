# Fricon v0.2 Product Direction

## Status

Supporting v0.2 product direction.

This is not current behavior. Read `design.md` first. Use this document for
user pain, user stories, and product-level detail after the canonical v0.2
model is clear.

## Product Positioning

Fricon v0.2 should be a local lab data library and automation foundation for
experimental science.

It should help experimentalists:

- explore quickly from Python scripts and notebooks
- record measurement history without manual folder discipline
- track samples, cooldowns, parameters, code, and datasets together
- visualize sample parameters and measurement results
- maintain large parameter sets without silent drift
- run repetitive calibration work with explicit proposals and history
- recover historical context when data, code, or parameters have evolved

Fricon should not make users operate a complex LIMS, hosted service, or
multi-user lab administration system before they can collect data.

Current v0.2 replacement target: record measurement metadata and produced
datasets for new measurement work. Users should be able to stop using a simple
LabRAD Grapher/Data Vault style logger for new measurements. Importing or fully
browsing legacy LabRAD/Data Vault history is a later migration workflow, not a
v0.2 requirement.

## Main User Pain

The current lab pattern Fricon should replace is:

```text
new sample or setup
  -> create a new data-vault folder
  -> sometimes copy a measurement code directory
  -> edit JSON parameters locally
  -> run scripts and save data
  -> later struggle to know which sample, cooldown, code, and parameters
     produced which result
```

This creates fragmented data history, duplicated code, drifting parameters, and
weak calibration provenance.

Fricon v0.2 should replace that pattern with:

```text
one Fricon data library
  -> sample and session records
  -> measurements
  -> dataset artifacts
  -> code and environment summaries
  -> parameter snapshots and proposals
  -> analysis and calibration history
```

## Reference-System Lessons

The useful lesson from LabRAD Data Vault/Grapher is speed: users can create
data, watch plots, and keep working without ceremony. Fricon should preserve
that low-friction loop while replacing folder/path conventions with explicit
data-library, sample/session, measurement, and provenance records.

The useful lesson from QCoDeS is separation of concerns: measurement code,
datasets, and broader experiment grouping are separate concepts. Fricon should
keep Python-led acquisition ergonomic, but make active context and stored
provenance visible so users are not surprised by hidden defaults.

The useful lesson from Labber is workflow completeness: measurement setup,
sweep structure, live/history browsing, comments, tags, and instrument-server
boundaries all matter to experimentalists. Fricon should learn from those user
flows without making `Log` the primary data noun or making a GUI-first runner
the only valid way to collect data.

## Primary User Model

The normal measurement flow should be:

```text
set active sample/session context when known
run measurement from Python
watch produced datasets
favorite important runs and review lifecycle flags
analyze in Python or UI
promote useful calibration results through parameter proposals
```

The first-screen product should eventually be organized around current lab work:

- active sample/session, with a visible "none selected" state
- recent measurements
- live datasets
- favorites, pins, and lifecycle flags
- parameter/calibration status

Dataset browsing remains important, but datasets are outputs inside a broader
measurement history.

## Core User-Visible Concepts

### Data Library

The user should normally have one main Fricon data library, not many workspaces.
The library is the local data root and catalog.

Samples, sessions, measurements, favorites, search, saved views, and optional
tags should organize the library without encouraging users to split data and
copied code into many long-lived roots.

Do not make `Project` or `Campaign` a first-class required v0.2 grouping. If
those ideas prove useful, introduce them later as lightweight grouping or saved
view concepts rather than as a required hierarchy.

Each data library should have a generated UUID and a user-editable display
name. Exported measurements should include that source identity so users can
tell which lab computer or data library produced the data.

### Sample

A sample is the measured physical object, device under test, wafer, chip, batch,
preparation, or specimen.

Samples need:

- stable identity
- display name and aliases
- structured custom fields
- favorites, notes, optional tags, and lifecycle state
- links to sample sessions, measurements, datasets, and parameter history
- optional 2D layout or coordinate map

Do not force all labs into a full inventory system. Sample records should be
lightweight but real.

Sample selection should be low-friction. Many existing notebooks set a
`data_dir` or similar global variable near the top of the file. Fricon should
support an equivalent active sample/session context in a notebook prelude and
in Fricon Desktop. Quick measurements may run without sample context and attach
or correct it later.

### Sample Session / Cooldown

A sample session records a period or condition under which a sample is used:

- cooldown cycle
- mounting or wiring configuration
- room-temperature characterization session
- wafer probing session
- campaign under a particular setup

The same sample may have multiple sessions. Parameter drift across cooldowns
belongs on sessions and parameter history, not usually on separate sample
identities.

Create a new sample only when the physical identity has meaningfully changed.
If processing creates a new object, record lineage from the old sample to the
new one rather than hiding the change in names or folders.

### Measurement

A measurement is the default record for data-taking work.

It should link:

- sample and session
- parameter snapshot or legacy parameter JSON
- code and environment summary
- produced datasets
- favorite or pin state, notes, optional tags, lifecycle flags, and attachments
- continuation or recovery decisions

Use `Experiment` for informal scientific discussion, or later for a broader
campaign/template/grouping if an ADR proves that users need that extra layer.
Do not make `ExperimentRun` the first public noun for the v0.2 API.

### Dataset Artifact

A dataset is a data artifact produced or consumed by work. It can be measured,
processed, imported, or simulated.

Datasets must remain directly openable from Python and Fricon Desktop, but they
should not be the only organizing object.

Reserve `Artifact` as the broader provenance concept. `DatasetArtifact` is the
primary v0.2 artifact because measured tables and live plots are the LabRAD
replacement path. Reports, figures, logs, attachments, waveform/configuration
files, code summaries, and future device snapshots should not have to masquerade
as datasets.

### Parameter Profile And Snapshot

Large parameter sets should be versioned explicitly.

Fricon should support:

- mutable refs or profiles for current working parameter sets
- immutable snapshots for run facts
- diffs between snapshots
- proposals from analysis or calibration
- validation and explicit promotion to refs

Calibration should not silently mutate important profiles during measurement.

### Code And Environment Summary

Fricon should reduce the need to copy code directories when sample or measurement
context changes.

Start with passive summaries:

- entry point or script path
- Git commit, dirty state, or file hash summary when available
- Python/fricon versions
- lock file or environment summary when practical

Do not make v0.2 a full Git client or environment manager.

## Sample Parameter Visualization

Sample visualization should be treated as a first-class product need.

v0.2 should design the sample model so custom fields and 2D visualization fit
cleanly, even if the initial implementation ships after the minimal measurement
loop.

User story:

> As an experimentalist, I want to define sample parameters and visualize them
> on a 2D layout so that I can choose measurement targets, compare device
> variation, and understand drift across sessions.

Capabilities to design toward:

- JSON/table editor for sample fields
- typed custom fields where useful
- 2D coordinate map or layout
- color by parameter, measurement result, favorite/lifecycle state, or
  calibration state
- link plotted points to measurements and datasets
- compare values across sessions/cooldowns
- show drift or history for selected sample points

This is likely as important as generic dataset browsing for many labs.

## v0.2 Must-Have User Stories

### Install And Launch Fricon

As an experimentalist, I want one clear way to install and launch Fricon on a
lab computer so that I can open Fricon Desktop, run the CLI, and use the Python
SDK against a local data library without assembling incompatible pieces by hand.

Acceptance notes:

- the supported v0.2 distribution shape is documented as Fricon Desktop,
  `fricon` CLI, Python SDK, and local service
- Fricon Desktop installs with a compatible local service sidecar; users should
  not install a separate server package for the normal local workflow
- Fricon Desktop, the bundled service, and the bundled CLI share one visible
  product release version
- Fricon Desktop starts in local mode for v0.2
- first-run setup creates or opens the default data library without requiring
  users to understand server internals
- the local service starts on demand from Fricon Desktop, CLI, or Python SDK
  where practical
- fixed lab computers may optionally enable a per-user "start Fricon service at
  login" mode, but root/system service registration is not required for
  ordinary v0.2 use
- notebook and script examples show how to connect to the same local data
  library used by Fricon Desktop
- users get a clear diagnostic when the local service is not running or cannot
  open the data library
- remote mode, browser-served UI, and PWA distribution are future-ready
  architecture targets, not v0.2 shipped workflows

### Update Without Breaking Measurement Work

As an experimentalist, I want Fricon updates to detect incompatible desktop,
CLI, Python SDK, local-service, or data-library versions so that a routine
update does not silently corrupt my measurement history.

Acceptance notes:

- the desktop app and local service may update together, but Python SDKs in
  lab scripts and notebooks often update more slowly because they are pinned by
  virtual environments or lockfiles
- Fricon Desktop can stage an update, but the local service reports whether it
  is safe to stop and replace
- updates are not applied while measurements, open dataset writers, imports,
  exports, or data-library migrations are active
- users can choose "install when idle" or "remind later" when an update is ready
  during active work
- "install when idle" may put the service into a draining state where existing
  work finishes, new long-running writes are blocked or warned, and read-only
  browsing continues where practical
- after v0.2 lands, the core v0.x Python SDK path for measurement writes and
  dataset reads should remain compatible with later v0.x local services
- newer capabilities are feature-negotiated rather than required by old scripts
- clients and the service negotiate protocol, API capability, and data-library
  format compatibility before writes
- Fricon Desktop, CLI, and Python SDK should use one public service API
  contract; do not keep Python on a separate gRPC public protocol while the UI
  moves to HTTP/WebSocket
- dataset writes should use explicit write sessions and binary Arrow-compatible
  payload chunks rather than row-by-row JSON or gRPC message-size-driven
  client complexity
- incompatible clients fail with a clear message and recovery path rather than
  partially writing data
- data-library format upgrades require explicit confirmation, no active
  measurements, and backup/checkpoint or recovery guidance where practical
- long-term third-party protocol stability and polished auto-update UX are
  follow-up topics, not v0.2 replacement requirements
- remote clients should never bypass the service by opening the same
  database-backed data library from a shared folder

### Create A Local Data Library

As an experimentalist, I want one Fricon data library so that data, sample
records, parameters, code summaries, and run history do not fragment into many
folders.

### Register A Sample

As an experimentalist, I want to create a lightweight sample record with custom
fields so that measurement data is tied to the object I measured.

### Start A Sample Session

As an experimentalist, I want to record a cooldown or measurement session for a
sample so that drift and context changes are tracked without pretending each
cooldown is a new sample.

The sample/session context should be selectable as an active default, not a
required modal step before every quick measurement.

### Run An Exploratory Measurement

As an experimentalist, I want to run a measurement from Python with minimal
boilerplate so that Fricon records the run, datasets, sample/session context,
and basic provenance.

If no sample/session is selected, Fricon should still record the measurement and
make missing context visible and fixable later.

### Watch And Inspect Data

As an experimentalist, I want local live and historical table/chart views so
that I can decide whether a measurement is working.

### Annotate Once At The Right Level

As an experimentalist, I want to favorite important measurements and add notes
or optional tags at the measurement or sample/session level so that I do not
have to annotate every dataset.

### Reopen Data From Python

As an analyst, I want stable IDs and read snippets so that I can reopen
measurement outputs without knowing storage paths.

### Export A Measurement For Offline Analysis

As an experimentalist, I want to export a complete measurement bundle so that I
can analyze it on another computer without setting up a Fricon data library or
importing the data first.

Acceptance notes:

- export starts from a measurement by default
- exported bundles include produced datasets, selected artifacts, favorites,
  notes, optional tags, lifecycle flags, sample/session context,
  parameter/code summaries, and provenance
- exported bundles include source data library UUID, display name, optional
  computer label, export UUID, format version, checksums, and original record
  IDs
- Python can open the bundle directly through a portable read API
- Fricon Desktop can open the bundle in a read-only export viewer mode
- importing the bundle into another data library is optional, not required for
  analysis

### Avoid Code Directory Copies

As an experimentalist, I want Fricon to record code and environment summaries
for runs so that I do not copy measurement code directories just to preserve
history.

Acceptance notes:

- v0.2 starts with passive summaries such as script path, Git/hash state,
  Python/Fricon versions, and environment hints when available
- managed code history and environment snapshots remain v0.3+ candidates

## v0.3/v0.4 Candidate User Stories

These stories are important for the long-term product, but they are not v0.2
replacement requirements. v0.2 should avoid blocking them in the data model.

### Track Parameter Evolution

As an experimentalist, I want parameter snapshots, diffs, and proposals so that
large parameter sets do not drift through untracked JSON edits.

### Visualize Sample Parameters

As an experimentalist, I want a 2D sample map colored by parameters or results
so that I can choose devices/regions and compare behavior across sessions.

### Calibrate With Reviewable Automation

As an experimentalist, I want scheduled or repeated calibration to produce
analysis results and parameter proposals so that tedious updates are automated
without silently mutating important parameter profiles.

### View Measurements From Another Computer

As an experimentalist, I want to view live and historical measurements from a
second lab computer so that another workstation can monitor data without
opening the data library through a shared folder.

Acceptance notes:

- this is a v0.3+ candidate, not a v0.2 replacement requirement
- future remote viewing should connect to the Fricon service that owns the data
  library
- Fricon Desktop remote mode may provide the most consistent app experience
- a browser-served read-only viewer may remain useful for quick access or
  troubleshooting

## v0.2 Design-Covered Edge Stories

These stories do not all need full UI workflows in v0.2. The v0.2 data model
should still avoid blocking them, because they are natural follow-ups for
v0.3/v0.4 once the basic measurement loop is usable.

### Attach Or Correct Context Later

As an experimentalist, I want to attach or correct sample/session context after
a measurement so that a forgotten active-context selection does not make the
data unusable.

Acceptance notes:

- missing context is visible, not treated as an error
- later attachment or correction records actor, time, old value, and reason
- corrected context affects browsing and filtering without rewriting measured
  dataset facts

### Recover From An Interrupted Measurement

As an experimentalist, I want an interrupted measurement to preserve partial
data and offer explicit continuation, invalidation, or new-measurement choices
so that failed hardware or script runs do not silently corrupt history.

Acceptance notes:

- partial datasets remain inspectable
- continuation requires explicit user or API intent
- continuation checks schema/semantic compatibility before appending
- interruption, invalidation, or supersession records a reason

### Compare Measurements And Sessions

As an experimentalist, I want to compare measurements across samples,
cooldowns, parameter snapshots, or code summaries so that drift and regressions
are visible without manually reconstructing history from folders.

Acceptance notes:

- comparison can start from sample, session, measurement, or dataset views
- parameter, code, favorite/lifecycle, and sample/session differences are
  visible beside plotted data
- saved comparisons should not create another data-library root

### Favorite Important Results And Track Lifecycle

As an experimentalist, I want favorites to highlight important results and
system lifecycle flags to show incomplete, interrupted, calibration/test,
invalidated, or superseded data so that later browsing and automation can avoid
bad inputs without requiring manual classification.

Acceptance notes:

- favorite/pin is the primary manual signal
- lifecycle flags come from execution, calibration/test context, or explicit
  invalidation/supersession actions
- optional notes/tags can explain unusual cases
- invalidation and supersession are events, not destructive rewrites

### Link Analysis Back To Source Data

As an analyst, I want analysis notebooks or scripts to record which
measurements and datasets they consumed and what results they produced so that
derived conclusions remain traceable.

Acceptance notes:

- analysis may produce derived datasets, reports, scalar results, or parameter
  proposals
- analysis outputs link to input datasets and measurements through provenance
- analysis is not stored as a child inside the original measurement
- Analysis is reserved in the model but should not be a peer navigation concept
  in the first measurement-facing UI

### Work On A Shared Lab Computer

As an experimentalist using a shared lab computer, I want Fricon to record a
lightweight actor label for mutating actions without forcing full account
management so that later history can explain who ran or changed something.

Acceptance notes:

- local single-owner mode remains the default product model
- actor labels/tokens are enough for audit summaries
- roles and permission matrices remain future scope

## Managed Measurement Framework Direction

Simple measurements should remain ordinary Python. Users should not need to
learn a declarative framework before they can collect data.

For repeated or automation-heavy measurements, Fricon may later provide an
optional managed measurement framework where users declare how parameter
snapshots, run-local inputs, and scan points resolve into desired device state
and dataset outputs.

User story:

> As an experimentalist with a repeated scan, I want to declare the intended
> device state for each scan point so that Fricon can preview the run, debug it
> with dummy devices, resume after interruption, and attach post-processing or
> calibration hooks.

The managed framework should help with:

- previewing the device state that will be applied before touching hardware
- inspecting differences between current device state and requested state
- using dummy or simulated devices for dry runs
- restart and resume after an interrupted scan
- explicit scan point identity and post-processing hooks
- safer automatic calibration workflows
- stronger provenance for device apply, readback, and output datasets

This should be a recommended integration path for managed runs, not the only
valid measurement style. Existing imperative measurement code should still be
able to record interactive runs and datasets, but advanced retry, resume,
dry-run, and automatic calibration behavior may require the managed
declarative API.

## Other v0.3+ Candidate User Stories

- Managed submitted measurements with queue and resource leases.
- Analysis-run UI for derived datasets and reports.
- Calibration workflow templates and history views.
- Import of legacy LabRAD/Data Vault history.
- Device identity and readback verification.
- AI-assisted metadata cleanup, reports, and calibration explanations.

## Product Non-Goals For v0.2

- Multi-user lab administration.
- Hosted SaaS.
- Full permissions/roles UI.
- Complete notebook state capture.
- Automatic code rewrite or environment management.
- Broad hardware driver framework.
- Generic workflow DAG engine as the first automation layer.
- Remote mode, browser-served UI, or PWA distribution.
- Shared-folder multi-machine access to the same database-backed data library.
