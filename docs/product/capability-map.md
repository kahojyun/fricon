# Capability Map

## Status

Draft rederived from current story map; older capability IDs retired.

## Purpose

Define the product capabilities implied by the current initial-adoption story
backbone without preserving older draft capability IDs. Capability names here
are planning labels, not implementation modules or stable requirements. Fresh
stable IDs can be added later when the rederived capability boundaries are
accepted enough to need traceability.

## Capability Rule

Use high-confidence inputs first:

- `vision.md`
- `personas.md`
- `story-map.md`
- the interview and challenge logs in `product-analysis-progress.md`

Do not promote older draft capability, epic, or user-story IDs into specs or
architecture. If a capability is not supported by the current story backbone,
it belongs in the backlog or should be rejected as old planning residue.

## First Usable Slice

The first usable slice should prove a maintained local replacement for the
basic Data Vault/Grapher loop around new interactive measurements:

```text
write from ordinary Python
  -> inspect simple live data
  -> keep checkpointed writes readable after ordinary interruption
  -> reopen by stable ID for analysis
```

Export, rich viewer behavior, managed execution, parameter systems, and device
communication are not prerequisites for this first slice.

## Initial Adoption Capabilities

### Local Product Readiness

Promise:

- Desktop, CLI, Python SDK, and required local runtime components behave like
  one coherent local product.

Includes:

- first-run setup
- local runtime and library diagnostics
- headless Python use when Desktop is closed
- fail-before-write compatibility checks
- support posture for common lab-computer constraints

Excludes:

- hosted service
- multi-user administration
- shared-folder multi-writer database semantics
- polished enterprise deployment

### Local Data Library

Promise:

- one normal lab computer can own a Fricon data library with durable identity
  and compatibility checks.

Includes:

- create/open
- remembered recent libraries
- library identity and format version
- locked, unsupported, or migration-required diagnostics

Excludes:

- distributed database semantics
- direct concurrent editing from shared folders
- old-history migration as a prerequisite

### Python Measurement Recording

Promise:

- an ordinary Python script or notebook can explicitly create an unmanaged
  measurement and append data with low ceremony.

Includes:

- measurement title and lifecycle
- concurrent local measurement writers
- unmanaged execution labeling
- measurement-scoped dataset writers
- trace-valued, array-valued, and generic step-record writes from the unmanaged
  path
- simple recording-section rewrites for Data Vault-style scripts

Excludes:

- managed runner
- scheduler or task queue
- visual sweep builder
- LabRAD compatibility layer
- device control

### Dataset Artifacts And Shape Semantics

Promise:

- produced data is recorded as directly openable dataset artifacts with enough
  shape semantics for simple plotting and later reading.

Includes:

- regular one-, two-, and N-dimensional sweeps
- trace-valued records with explicit coordinate/value arrays
- compact regular-coordinate trace descriptions
- multiple traces in one outer sweep record
- first-class complex values
- IQ average, I/Q channel, single-shot array, or label data
- generic irregular/ragged step records
- missing/partial shape semantics where meaningful

Excludes:

- forcing every acquisition into a regular grid
- treating minimizers as a special first-slice product model
- making internal storage streams the normal user concept

### Simple Live Inspection

Promise:

- Desktop can watch active data without slowing or breaking acquisition writes.

Includes:

- live line/scatter views
- simple heatmap
- selected output or trace channel
- simple magnitude/phase or I/Q view from data semantics
- IQ scatter
- stale, lagging, or disconnected view indicators
- multiple visible measurement or monitor views where practical

Excludes:

- live consumers as write acknowledgements
- row-specific analysis controls in the live monitor
- running user plotting code
- fitting, classifier tuning, or publication plotting
- full viewer behavior as a real-time surface

### Checkpoint-Safe Readability

Promise:

- already-written measurement data remains readable after ordinary interruption
  cases.

Includes:

- incomplete lifecycle states
- checkpointed data reads after user interrupt or notebook-kernel failure
- visible missing or partial points where schema supports them
- trash/recover for ordinary cleanup

Excludes:

- hard-crash or power-loss recovery promises beyond accepted durable-write
  behavior
- resuming unmanaged Python execution from the last scan point
- making incomplete data look complete

### Stable-ID Reopen

Promise:

- users can reopen measurements and dataset artifacts from Python by stable ID
  without depending on private storage paths.

Includes:

- stable measurement and dataset IDs
- fast ID copy from Desktop or CLI
- predictable public reader entry point
- simple reader output that users can wrap in experiment-specific helpers
- schema, unit, label, lifecycle, and partial-data metadata
- local reads into common Python objects where appropriate

Excludes:

- private storage layout as public API
- committing the internal storage model to Polars, pandas, NumPy, xarray, or
  any other analysis framework
- polished alternate framework-specific views as a first-slice blocker

### Dataset Browser And Direct Open

Promise:

- dataset artifacts remain searchable and directly openable even though Desktop
  is measurement-first.

Includes:

- direct open from measurement or dataset context
- parent measurement context in dataset views
- table and plot previews where shape supports them
- right-click or keyboard shortcut ID copy

Excludes:

- returning to a dataset-first Desktop home
- treating internal streams as first-class navigation targets

### Lightweight Notes, Attributes, And Attachments

Promise:

- users can attach practical human context without turning Fricon into a full
  ELN or setup truth system.

Includes:

- measurement and dataset notes
- markers, favorites, pins, and optional tags
- user-supplied attributes
- small attachments
- source metadata for attachments where practical
- correction events

Excludes:

- full electronic lab notebook replacement
- attachment role taxonomy as a first-slice requirement
- broad report or presentation generation

### Honest Software-Visible Context

Promise:

- Fricon records context it can honestly know, or that the user explicitly
  supplies, without pretending it owns execution or lab setup truth.

Includes:

- unmanaged code labels
- optional script path or source-folder reference
- selected local configuration files
- parameter or registry files as attachments/evidence
- demod/readout settings when supplied
- environment labels or summaries
- unmanaged procedure summaries
- privacy-aware handling of local details

Excludes:

- automatic notebook state capture
- parsing arbitrary parameter, registry, setup, or wiring files into trusted
  Fricon-owned facts
- judging opaque context as fresh, stale, trusted, or ambiguous without
  explicit evidence
- managed code snapshots
- parameter profiles or proposal workflows
- device inventory or control

### Migration Guidance For New Work

Promise:

- users can migrate new measurement scripts gradually without rewriting the
  whole experiment stack.

Includes:

- Data Vault-style recording-section rewrite guidance
- mapping independent/dependent declarations to Fricon dataset shapes
- examples for VNA traces, coarse/fine trace collections, first-class complex
  values, IQ-like data, and generic irregular records
- legacy aliases as metadata, not primary identity
- old history left in the old system

Excludes:

- built-in old-history import
- LabRAD server emulation
- LabRAD-dependent helper module
- Data Vault browser

## Follow-On Backlog

These capabilities are useful, but should not block the first usable slice:

- portable Fricon package export
- generic CSV, Parquet, NumPy, or other file exports
- offline bundle reader polish and read-only bundle viewer
- rich historical viewer, saved views, and comparison workflows
- trace concatenation helpers and coarse/fine overlay polish
- polished Polars, pandas, NumPy, or xarray-specific reader adapters
- analysis result records, fit records, interpretation records, and report
  artifacts
- parameter profiles, effective snapshots, diffs, proposals, and reviewed
  promotion
- managed code sources, approved releases, environment locks, and managed
  execution
- calibration records, calibration chains, health gates, and reviewed
  automation
- structured setup/device state, communication, reconciliation, and apply
- run manifests and failure investigation views
- remote or LAN monitoring
- rich sample maps and spatial visualization
- AI-assisted reviewed actions

## ADR-Gated Directions

These directions need explicit ADRs before durable product scope:

- device communication or mutation
- setup/device apply or desired-state reconciliation
- mutation-capable calibration automation
- resumable execution checkpoints
- distributed or shared editable data libraries
- compatibility promises for third-party protocols or legacy formats

## Rejected For First Adoption

- hosted SaaS
- account/team administration
- LabRAD compatibility server
- built-in legacy Data Vault import or browser
- broad device-driver framework
- generic workflow DAG engine
- automatic notebook state capture
- hard-crash recovery claims beyond accepted durable-write behavior
- treating physical setup notes as Fricon-owned truth
