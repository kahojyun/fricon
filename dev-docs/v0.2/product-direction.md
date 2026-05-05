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
- track samples, sessions/cooldowns, parameters, code, and datasets together
- visualize sample parameters and measurement results
- set up and reuse measurement code across lab computers without copying
  folders by hand
- maintain large parameter sets without silent drift
- leave room for repetitive calibration work with explicit proposals and
  history
- recover historical context when data, code, or parameters have evolved

Fricon should not make users operate a complex LIMS, hosted service, or
multi-user lab administration system before they can collect data.

Current v0.2 replacement target: record measurement metadata and produced
datasets for new measurement work. Users should be able to stop using a simple
LabRAD Grapher/Data Vault style logger for new measurements. Fricon should
provide generic APIs that users can use to write migration scripts for old data
when needed, but direct built-in import or full browsing of legacy LabRAD/Data
Vault history is not a v0.2 product commitment.

## Main User Pain

The current lab pattern Fricon should replace is:

```text
new sample or setup
  -> create a new data-vault folder
  -> sometimes copy a measurement code directory to the same or another lab PC
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
  -> code provenance and environment summaries
  -> parameter snapshots and proposals
  -> analysis and calibration history
```

The first v0.2 implementation should not try to deliver every item in that
target model. v0.2 should prove recording new measurements and datasets first;
analysis records, automatic calibration, and managed device communication are
later layers that the model must not block.

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
analyze in Python or exported bundles
later promote useful calibration results through parameter proposals
```

The first-screen product should eventually be organized around current lab work:

- active sample/session, with a visible "none selected" state
- recent measurements
- live datasets
- favorites, pins, and lifecycle flags
- later parameter/calibration status

Dataset browsing remains important, but datasets are outputs inside a broader
measurement history.

Measurement identity in the UI should be easy to read without memorizing file
paths or numeric data-vault folders. Use the measurement name/title, start time,
and sample/session label when available as the primary display identity, with
stable IDs available as secondary technical references.

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

The preferred path is saved views and tags first. Add `Project` or `Campaign`
only if real workflows show that filters and views cannot carry the grouping
need.

Each data library should have a generated UUID and a user-editable display
name. Exported measurements should include that source identity so users can
tell which lab computer or data library produced the data.

The ordinary v0.2 experience should optimize for one primary local data library
per machine. Opening or switching to another library can exist as an advanced
or diagnostic workflow, but frequent project-style library switching should not
be the default model.

### Sample

A sample is the measured physical object, device under test, wafer, chip, batch,
preparation, or specimen.

Samples need:

- stable identity
- display name and aliases
- flexible typed custom fields
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

Use a generic `Sample Session` or `Use Session` concept with an optional session
type such as cooldown, mounting, treatment, probing, or campaign. Do not make
`Cooldown` the only first-class session noun.

Create a new sample only when the physical identity has meaningfully changed.
If processing creates a new object, record lineage from the old sample to the
new one rather than hiding the change in names or folders.

### Measurement

A measurement is the default record for data-taking work.

It should link:

- sample and session
- parameter snapshot or legacy parameter JSON
- optional instrument/setup/method labels
- code and environment summary
- produced datasets
- favorite or pin state, timestamped notes/markers, optional tags, lifecycle
  flags, and light attachments
- basic clock/timing source metadata for event ordering, especially on offline
  or locked-down lab computers
- continuation or recovery decisions

Use `Experiment` for informal scientific discussion, or later for a broader
campaign/template/grouping if an ADR proves that users need that extra layer.
Do not make `ExperimentRun` the first public noun for the v0.2 API.

Python SDK examples should create measurements explicitly but with little
ceremony. Dataset writers should share the measurement lifecycle so users do
not have to nest a separate writer context for every output dataset.

### Dataset Artifact

A dataset is a data artifact produced or consumed by work. It can be measured,
processed, imported, or simulated.

Datasets must remain directly openable from Python and Fricon Desktop, but they
should not be the only organizing object.

Focused dataset artifact product requirements and user stories live in
`dataset-artifact-requirements.md`. Treat that note as the product-level source
for dataset behavior before writing storage, API, chart, or export ADRs.

Dataset contents should be appendable while their writer is active and
immutable after the producing measurement finishes. Corrections should create
derived artifacts or correction records rather than silently editing completed
dataset facts in place.

Reserve `Artifact` as the broader provenance concept. `DatasetArtifact` is the
primary v0.2 artifact because measured tables and live plots are the LabRAD
replacement path. Reports, figures, logs, attachments, waveform/configuration
files, code provenance summaries, managed code snapshots, and future device
snapshots should not have to masquerade as datasets.

### Parameter Profile And Snapshot

Large parameter sets should be versioned explicitly.

v0.2 may start with an optional flexible parameter snapshot, such as JSON-like
metadata attached to a measurement. A full global parameter registry, profile
editing UI, and calibration promotion workflow are later scope.

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

- user-provided code label and entry point or script path when available
- optional Git commit, dirty state, or file hash summary when available
- Python/fricon versions
- lock file or environment summary when practical

Do not make v0.2 a full Git client or environment manager.

### Measurement Code Source

A measurement-code source is the lab-managed source of acquisition code and
reusable measurement assets for one or more lab computers.

The concept should help with the real workflow where several equipment PCs are
used by different groups and code is copied between them. Fricon should separate
three things that were previously mixed together:

- the local Fricon data library on each acquisition computer
- the local checkout or installed package used to run measurement code
- the upstream source, such as a self-hosted Gitea repository, GitLab/GitHub
  repository, bare Git mirror, read-only network mirror, or package cache

v0.2 should record honest source summaries and surface diagnostics. For
non-managed user-run Python, that may only be `unmanaged` plus optional
user-supplied labels. For future managed runs, Fricon can require a resolved
immutable code snapshot before execution. A future UI can make setup and update
easier, but the data model should avoid assuming copied folders are the normal
preservation strategy.

High-value reusable assets include:

- measurement scripts and helper modules
- scan-schema helpers and measurement templates
- plot presets and display defaults
- setup profiles with machine-local overrides
- environment lock files or package constraints
- export recipes
- future calibration workflow definitions

Keep these local or machine-specific unless explicitly exported:

- raw data libraries
- active sample/session state
- local service tokens and secrets
- device addresses, ports, and safety limits that differ by setup
- dirty local code changes and temporary notebook state

Do not make network storage the active shared data library or the primary
editable code folder. It can be a practical mirror, package cache, backup
target, or export destination.

Managed execution should use the code source differently from ordinary
interactive scripts:

```text
configured Git/Gitea source
  -> local bare mirror or cache
  -> immutable commit or tree snapshot
  -> temporary execution worktree
  -> ScriptRun
  -> Measurement
```

This gives future script runner, managed measurement, and calibration workflows
stronger provenance without requiring every exploratory notebook to become a
managed run.

## Sample Parameter Visualization

Sample visualization should be treated as an important follow-up product need.

v0.2 should design the sample model so custom fields and 2D visualization fit
cleanly, even if the initial implementation ships after the minimal measurement
loop.

The sample map should be a strong v0.3+ secondary view and filter entry, not a
first-slice v0.2 home screen. The measurement console remains the default home.

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
lab computer so that I can open Fricon Desktop and use the Python SDK against a
local data library without assembling incompatible pieces by hand.

Acceptance notes:

- Windows is a first-class v0.2 target because lab computers are expected to be
  Windows machines
- macOS is also supported for development and normal local use
- the supported v0.2 distribution shape is documented as Fricon Desktop,
  `fricon` CLI, Python SDK, and local service
- the CLI is bundled for setup, diagnostics, service control, and developer
  workflows; normal experimental workflows should rely on Desktop and Python
- Fricon Desktop installs with a compatible local service sidecar; users should
  not install a separate server package for the normal local workflow
- Fricon Desktop, the bundled service, and the bundled CLI share one visible
  product release version
- Fricon Desktop starts in local mode for v0.2
- first-run setup creates or opens the default data library without requiring
  users to understand server internals
- first-run setup asks for the data-library location and remembers it
- the local service starts on demand from Fricon Desktop, CLI, or Python SDK
  where practical
- fixed lab computers may optionally enable a per-user "start Fricon service at
  login" mode, but root/system service registration is not required for
  ordinary v0.2 use
- the local service uses a generated local token boundary instead of relying on
  unauthenticated open loopback writes
- notebook and script examples show how to connect to the same local data
  library used by Fricon Desktop
- setup guidance distinguishes Fricon installation, local data-library
  location, measurement-code source, and Python environment as separate things
- the recommended Python SDK setup is normal lab-environment installation, such
  as `uv` or `pip`, plus service discovery/startup diagnostics
- users get a clear diagnostic when the local service is not running or cannot
  open the data library
- Python measurement scripts can run headlessly without first opening Fricon
  Desktop
- when a Python script starts a measurement, an already-open Desktop highlights
  the new live run in the console without stealing focus or opening a window
  automatically
- remote mode, browser-served UI, and PWA distribution are future-ready
  architecture targets, not v0.2 shipped workflows

### Record And Diagnose Code Provenance

As an experimentalist, I want Fricon to record whether a measurement came from
unmanaged code, user-supplied code context, or a future managed snapshot so
that copied folders do not create false reproducibility.

Acceptance notes:

- v0.2 records the code provenance level for each measurement when available,
  but it does not need to implement full code installation or update workflows
- setup docs and diagnostics treat Fricon install, data-library location,
  measurement-code source, and Python environment as separate checks
- code runs from a local checkout, installed package, or local environment on
  the measurement computer
- a self-hosted Gitea repository, shared Git repository, read-only network
  mirror, or package cache can be an upstream source, but Fricon does not
  require a central Fricon server
- network storage may be used as a mirror/cache/export/backup target, not as
  the active database-backed data library or main editable code workspace
- future setup UI can expose "install approved code", "update to approved
  release", "show what changed", "run environment check", and "export local
  changes for review" actions without requiring ordinary users to operate Git
  directly

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
- update checks are prompted; users choose when to install instead of Fricon
  silently replacing the running measurement environment
- updates are not applied while measurements, open dataset writers, imports,
  exports, or data-library migrations are active
- users can choose "install when idle" or "remind later" when an update is ready
  during active work
- "install when idle" may put the service into a draining state where existing
  work finishes, new long-running writes are blocked or warned, and read-only
  browsing continues where practical
- after v0.2 lands, recorded data durability matters more than preserving every
  v0.x SDK, CLI, UI, or protocol API shape
- v0.x APIs and protocol details may break when needed, but incompatible
  clients must fail before writes with clear upgrade or migration guidance
- newer capabilities are feature-negotiated when practical rather than failing
  ambiguously in old scripts
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
- v0.2 includes a basic built-in backup/restore path for the local data library;
  migration and repair create checkpoints where practical before making changes
- long-term third-party protocol stability and polished auto-update UX are
  follow-up topics, not v0.2 replacement requirements
- remote clients should never bypass the service by opening the same
  database-backed data library from a shared folder

### Create A Local Data Library

As an experimentalist, I want one Fricon data library so that data, sample
records, parameters, code provenance summaries, and run history do not fragment
into many folders.

Acceptance notes:

- first-run setup asks where the data library should live and remembers the
  choice
- Fricon should not silently choose a hidden app-data path for experimental
  data unless the user explicitly accepts it
- data libraries have generated UUIDs and user-editable display names
- basic manual backup/restore is part of the data-library UX, not only an
  internal migration safety step
- migration or repair requires the service to be idle and should create a
  backup/checkpoint where practical

### Register A Sample

As an experimentalist, I want to create a lightweight sample record with custom
fields so that measurement data is tied to the object I measured.

Acceptance notes:

- sample fields are flexible and typed where useful, not a strict lab-wide
  inventory schema
- Fricon Desktop supports inline creation, selection, and basic editing during
  measurement setup
- a full sample database manager and rich sample-map editor can follow after
  the minimal measurement loop

### Start A Sample Session

As an experimentalist, I want to record a cooldown or measurement session for a
sample so that drift and context changes are tracked without pretending each
cooldown is a new sample.

The sample/session context should be selectable as an active default, not a
required modal step before every quick measurement.

Acceptance notes:

- use a generic sample session with an optional type label such as cooldown,
  mount, treatment, probing, or campaign
- users can attach or correct sample/session context later with change history
- Fricon keeps active context visible but does not rely on unreliable
  stale-context warnings as the main safeguard
- users can select recent measurements and bulk-correct sample/session context
  with history when active context was wrong

### Run An Exploratory Measurement

As an experimentalist, I want to run a measurement from Python with minimal
boilerplate so that Fricon records the run, datasets, sample/session context,
and basic provenance.

If no sample/session is selected, Fricon should still record the measurement and
make missing context visible and fixable later.

Acceptance notes:

- the Python SDK uses an explicit but short measurement-creation call
- measurement-scoped dataset writers finalize with the measurement lifecycle
- optional flexible parameter snapshots can be attached without a global
  parameter registry
- multiple local measurement writers may be active concurrently through the
  service, each isolated as its own measurement
- when a script crashes, the partial measurement remains visible as partial or
  failed; a rerun creates a new linked measurement by default instead of
  silently appending to the old one
- v0.2 targets Grapher-plus scale: comfortably beyond simple LabRAD Grapher
  replacement workloads, with storage and API choices ready for larger later
  versions

### Watch And Inspect Data

As an experimentalist, I want local live and historical table/chart views so
that I can decide whether a measurement is working.

Acceptance notes:

- the first screen should be a measurement console, not a generic dataset
  browser
- the console centers active and recent measurements, active sample/session
  context, produced datasets, and live status
- replacing LabRAD Grapher requires both live monitoring and historical
  browsing/recovery
- v0.2 should support live table and plot views for table-shaped measurement
  datasets
- core plot scope is table, line/scatter, and basic 2D heatmap/image views from
  tabular columns; richer dashboards wait for later versions
- live plots are operational instruments for deciding whether a run is sane,
  not publication-figure tooling
- live plotting, previews, export preparation, and future analysis hooks are
  noncritical consumers; they must not slow or fail acquisition writes
- interrupted or partial measurements remain visible and recoverable
- users can detach measurement or plot windows from the main console to watch
  multiple active runs without creating multiple full app instances
- default shortcuts include today, live/active, active sample/session,
  favorites, partial/failed, trash, and text search
- deeper history views may support structured filters by measurement type,
  lifecycle state, and dataset columns where practical

### Diagnose Local Setup Problems

As an experimentalist, I want Fricon Desktop to explain setup and connection
problems so that I can fix common lab-computer issues without reading raw logs.

Acceptance notes:

- diagnostics cover stopped service, missing service, wrong data-library
  location, locked data library, old Python SDK, incompatible service, pending
  update, and migration-required states
- diagnostics provide user-facing next steps before pointing to logs
- support bundles are local and redacted by default; users explicitly choose
  when to export diagnostic details
- CLI diagnostics may exist for power users, but Desktop diagnostics are the
  primary v0.2 support surface

### Annotate Once At The Right Level

As an experimentalist, I want to favorite important measurements and add notes
or optional tags at the measurement or sample/session level so that I do not
have to annotate every dataset.

Acceptance notes:

- users can add basic timestamped notes or markers during and after a
  measurement
- notes and markers appear in a measurement event timeline beside lifecycle and
  system events, rather than only in one static note field
- metadata, sample/session links, notes, tags, and lifecycle flags are
  correctable with history for important changes
- a lightweight optional local operator profile can label mutating actions
- the operator/profile label is a machine or session default, not a prompt for
  every measurement
- tags and notes remain optional; favorites/pins are the primary manual signal

### Reopen Data From Python

As an analyst, I want stable IDs and read snippets so that I can reopen
measurement outputs without knowing storage paths.

Acceptance notes:

- snippets use the public Python SDK, not internal storage paths
- examples assume the SDK was installed in the user's lab Python environment
  through `uv`, `pip`, or equivalent environment tooling
- connection discovery should find or start the local service where practical
  and fail with guided diagnostics otherwise

### Export A Measurement For Offline Analysis

As an experimentalist, I want to export a complete measurement bundle so that I
can analyze it on another computer without setting up a Fricon data library or
importing the data first.

Acceptance notes:

- export starts from a measurement by default
- exported bundles include produced datasets, selected artifacts, favorites,
  notes, optional tags, lifecycle flags, sample/session labels, source data
  library UUID/display name, export UUID, format version, checksums, original
  record IDs, Fricon versions, setup/method labels, and basic timing metadata
  by default
- exports include convenient common tabular files, such as CSV or Parquet when
  practical, in addition to the Fricon bundle manifest
- exports include a simple human-readable manifest or index preview; do not
  present this as a full analysis report
- exports are analysis packages, not raw file dumps; include enough IDs, units,
  timing, notes/events, sample/session, checksums, and Python loader snippets
  for laptop or HPC analysis
- sensitive provenance such as full code paths, dirty Git details, full
  environment summaries, source computer label, and extensive sample metadata
  should be opt-in or explicitly previewed before export
- Python can open the bundle directly through a portable read API
- Fricon Desktop can open the bundle in a read-only export viewer mode without
  requiring a running local service
- importing the bundle into another data library is optional, not required for
  analysis

### Trash And Recover Data

As an experimentalist, I want to move mistaken test measurements or datasets to
trash so that ordinary cleanup does not permanently destroy scientific data.

Acceptance notes:

- v0.2 normal UX exposes trash/recover, not hard delete
- completed measurements and datasets should remain recoverable from normal
  cleanup actions
- permanent deletion can remain an internal maintenance or future advanced
  operation, not a primary v0.2 user workflow
- correction, invalidation, and supersession should be represented as events
  rather than destructive rewrites

### Avoid Code Directory Copies

As an experimentalist, I want Fricon to distinguish unmanaged code from managed
code snapshots so that I do not copy measurement code directories just to
preserve history or create false reproducibility.

Acceptance notes:

- non-managed Python measurements should be allowed to record measurement data,
  but code provenance should be `unmanaged` unless the user explicitly provides
  a label or summary
- Fricon should not automatically inspect arbitrary notebooks, imports, or
  working trees and present that as reproducible history
- managed measurements can require a code snapshot resolved from a configured
  source before execution
- managed summaries should include the source label, repository identity,
  release/tag/commit, tree hash, dirty state, entry point, and environment
  hints when available
- the summary should make it visible when a lab computer is running unknown,
  dirty, locally modified, or unmanaged measurement code
- full source capture and environment snapshots remain v0.3+ candidates tied
  to the managed runner/code-source design

### Write Table-Shaped Measurement Data

As an experimentalist, I want v0.2 to reliably record table-shaped measurement
data so that the replacement workflow is predictable before broader artifact
types are added.

Acceptance notes:

- table-shaped datasets are the official v0.2 data shape
- numeric and complex-valued measurement columns are in scope
- datasets intended for live or historical plotting require explicit scan
  schema at creation time
- scan schema should include roles such as setpoint/independent,
  measured/dependent, fixed/config, monitor/readback, and enough axis shape or
  ordering metadata for slicing and display
- scratch or unplotted tables may use a generated guessed schema
- column unit, label, and display hints are lightweight metadata that complement
  scan schema
- light measurement attachments such as small files, images, and logs are in
  v0.2 scope; rich file/artifact management, dense arrays, waveform files,
  reports, and broader artifact workflows can follow later

## v0.3/v0.4 Candidate User Stories

These stories are important for the long-term product, but they are not v0.2
replacement requirements. v0.2 should avoid blocking them in the data model.

The first likely v0.3 product slice is read-only LAN viewing from another
computer, because it matches an existing LabRAD Grapher usage pattern while
preserving the single-owner data-library model.

### Track Parameter Evolution

As an experimentalist, I want parameter snapshots, diffs, and proposals so that
large parameter sets do not drift through untracked JSON edits.

### Visualize Sample Parameters

As an experimentalist, I want a 2D sample map colored by parameters or results
so that I can choose devices/regions and compare behavior across sessions.

Acceptance notes:

- sample map is a secondary view and filter entry, not the default home screen
  in v0.2

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
- the first remote phase is strict read-only monitoring, browsing, and export;
  remote annotations and acquisition writes remain later scope
- Fricon Desktop remote mode may provide the most consistent app experience
- a browser-served read-only viewer may remain useful for quick access or
  troubleshooting

### Install Or Update Shared Measurement Code

As an experimentalist, I want Fricon to help install or update approved
measurement code on a lab computer so that different equipment PCs do not drift
through copied folders and ad hoc local edits.

Acceptance notes:

- this is a v0.3+ candidate, not a v0.2 replacement requirement
- Fricon may wrap an existing Gitea, GitLab, GitHub, bare Git mirror, read-only
  network mirror, package cache, or lab-managed release bundle rather than
  hosting code itself
- ordinary users should see approved releases, current local version,
  environment status, and a change summary, not raw branch-management UI
- maintainers can still use normal Git and environment tools outside Fricon
- managed runners may maintain local bare mirrors or caches and expand
  immutable snapshots into temporary execution worktrees
- Fricon should warn when local changes exist before updating or running a
  measurement
- machine-specific setup profiles, device addresses, secrets, and local
  overrides stay local

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
cooldowns, parameter snapshots, or code provenance summaries so that drift and
regressions are visible without manually reconstructing history from folders.

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
- v0.2 analysis remains external through Python/export; managed analysis records
  and result UI are later scope

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

Long-term acquisition remains Python-first. Fricon can add preview, inspect,
and managed-plan helpers, but a Labber-like visual sweep builder is not a
current product goal.

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
- Lightweight measurement templates for repeated names, scan schemas, and
  display defaults after the explicit scan-schema path proves useful.
- Passive setup/device snapshots with software, firmware, driver, method, and
  calibration-state summaries.
- Guided lab-computer setup that installs/checks Fricon, chooses a local data
  library, connects to an approved measurement-code source, checks the Python
  environment, selects a setup profile, and runs diagnostics.
- Measurement-code source manager for approved releases/tags, update checks,
  local-change warnings, and maintainer handoff.
- Compare-what-changed views across sample, setup/method labels, parameters,
  code, operator, calibration state, and produced data.
- Rerun-from-artifact workflow that starts from a previous measurement's scan
  schema, labels, code/method summary, and display defaults.
- Offline or silent installer, rollback, and side-by-side version support for
  locked-down Windows lab PCs.
- Richer search by setup, operator, parameter, method/config, instrument label,
  and calibration state.
- Generic import APIs and examples that let users write their own legacy-data
  migration scripts without Fricon depending on LabRAD/Data Vault internals.
- Device identity and readback verification.
- AI-assisted metadata cleanup, reports, and calibration explanations.

## Product Non-Goals For v0.2

- Multi-user lab administration.
- Hosted SaaS.
- Full permissions/roles UI.
- Complete notebook state capture.
- Automatic code rewrite or environment management.
- Full Git client, Git hosting, or source-control training UI for ordinary
  experimenters.
- Central Fricon server required only to distribute measurement code.
- Shared editable network folder as the primary measurement-code workflow.
- Broad hardware driver framework.
- Fricon-managed device communication.
- Automatic calibration workflows.
- Managed/declarative measurement framework.
- Labber-like visual sweep builder as a product goal before managed-plan
  previews prove a need.
- Driver marketplace or shared-driver ecosystem in v0.2.
- Regulated-lab compliance UX as a v0.2 product goal; use regulated workflows
  only as a traceability stress test.
- LabRAD Data Vault/Grapher compatibility layer for old scripts.
- Full legacy LabRAD/Data Vault import or browsing.
- Direct built-in Data Vault storage parser or importer.
- Generic workflow DAG engine as the first automation layer.
- Remote mode, browser-served UI, or PWA distribution.
- Shared-folder multi-machine access to the same database-backed data library.
