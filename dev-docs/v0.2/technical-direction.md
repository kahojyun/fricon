# Fricon v0.2 Technical Direction

## Status

Supporting v0.2 technical direction.

This is not current behavior and not an implementation plan. Read `design.md`
first. Use this document to align architecture discussions, ADRs, and early
spike work after the canonical v0.2 model is clear.

## Rewrite Strategy

Do not create a permanent separate `fricon-v2` project.

Recommended path:

```text
same repository
  -> v0.2 foundation branch / jj change stack
  -> focused spikes when needed
  -> replace core model in place
```

A short-lived prototype crate or scratch directory is acceptable for schema/API
experiments, but durable code should return to this repository so tests, docs,
packaging, release process, and migration decisions stay together.

## Expected Debt

The existing code is not uniformly bad, but the product model is shifting.

High rewrite risk:

- public workspace mental model
- dataset catalog schema
- dataset metadata ownership
- Python dataset creation API
- desktop dataset-first navigation
- IPC/protobuf contracts
- archive/import/export assumptions
- run, sample, parameter, code, and provenance boundaries
- setup assumptions that rely on copied code folders, shared editable network
  directories, or one-off local Python environments

Likely reusable or adaptable:

- Rust/Python/frontend build infrastructure
- Arrow payload IO concepts
- append-only writer/read path concepts
- record ID and semantic manifest ideas
- chart rendering and transform code after DTO/model changes
- Tauri packaging and web frontend stack
- testing, release, and documentation infrastructure

Treat the v0.2 rewrite as a domain-model reset, not a full repository rewrite.

## Public Distribution Surfaces

v0.2 should define three public surfaces:

```text
Fricon Desktop
fricon CLI
fricon Python SDK
```

They should operate against one local data-library service and storage model:

```text
Fricon Data Library
  local service / daemon
  storage
  API
  Fricon Desktop
  CLI
  Python SDK
```

The Python SDK remains a first-class surface for measurement scripts and
analysis notebooks. Fricon Desktop is the primary browsing, inspection, sample,
and live-measurement surface. The CLI should exist mainly for setup,
diagnostics, service control, and developer workflows, not as the broad
ordinary-user workflow surface.

v0.2 is local-only. The shipped GUI should be Fricon Desktop in local mode,
backed by the local service. Remote mode, browser-served UI, and static
PWA-style distribution are future compatibility targets, not v0.2 release
requirements.

## Installation And Update Experience

Installation and update are user-facing v0.2 concerns, even though their
implementation details are technical.

The product-level requirement is:

```text
Fricon distribution
  -> Fricon Desktop installer bundles the local service sidecar
  -> Fricon Desktop, local service, and bundled CLI share one product release
  -> Python SDK may be pinned in lab environments
  -> data-library format upgrades are explicit
```

Users should be able to install Fricon, create or open a data library, launch
Fricon Desktop, and connect from Python without understanding service
internals.

Windows is the first-class v0.2 lab-computer target. macOS remains supported
for development and normal local use. First-run setup should ask where the data
library should live and remember that choice.

Do not make users install the GUI and server separately in the default v0.2
flow. Fricon Desktop should ship with a compatible local service binary and a
compatible CLI. The service may be launched on demand by Fricon Desktop, the
CLI, or the Python SDK. A fixed lab computer may optionally enable a per-user
"start Fricon service at login" mode, but v0.2 should not require root/system
service registration for ordinary use.

Lab PCs may be locked down, offline, firewalled, or managed by IT. v0.2 should
avoid web-installer-only assumptions. Silent/offline installation,
side-by-side versions, and rollback are important follow-up packaging topics
once the local replacement workflow is proven.

Python measurement scripts should be able to run headlessly without first
opening Fricon Desktop. The Python SDK should discover, start, or connect to
the local service where practical and report guided diagnostics when it cannot.

New lab-computer setup should be treated as a product workflow, even if v0.2
only documents and diagnoses part of it. The user-visible pieces are:

- install Fricon Desktop, bundled service, and CLI
- choose or create the local data library
- connect the computer to the lab's measurement-code source when one exists
- create or select the Python environment used by measurement scripts
- select a local setup profile with machine-specific device bindings and
  overrides
- run diagnostics before the first measurement

v0.2 does not need to automate all of this. It should avoid architecture that
would make the future flow depend on a central Fricon server or on opening a
database-backed data library through a shared folder.

Use one visible product version for the bundled desktop/service/CLI release:

```text
Fricon Desktop 0.2.3
  bundles local service 0.2.3
  bundles compatible CLI 0.2.3
```

Keep separate internal compatibility versions where they matter:

- service API or protocol version
- data-library format version
- export bundle format version
- feature capabilities

The Python SDK package version may differ because lab scripts and notebooks can
pin it through virtual environments or lockfiles. Compatibility must be checked
through API/protocol/capability negotiation, not by requiring exact package
version equality.

Updates should be prompted and applied only when the local service says it is
safe. Fricon Desktop may drive the user prompt and installer, but the service
owns the busy/idle state. Do not silently auto-install updates on a measurement
computer.

Safe update flow:

```text
update available
  -> download or stage update
  -> ask service: safe_to_update?
  -> if idle: stop service, replace bundle, run required checks, restart
  -> if busy: offer "install when idle" or "remind later"
```

When users choose "install when idle", the service may enter a draining state:

- existing measurement writers, imports, exports, or maintenance jobs may
  finish
- new long-running writes or tasks are blocked or warned
- read-only browsing can continue where practical
- when no blockers remain, Fricon Desktop can restart the service and apply the
  update

v0.2 should track enough service activity to avoid unsafe updates:

- active measurement records
- open dataset writers
- active import or export work
- active data-library migration or repair
- future managed tasks, calibration jobs, and resource leases

Data-library format upgrades must be explicit. Do not silently migrate a data
library during app launch or while measurements are active. If an update
requires a data-library upgrade, Fricon should explain the change, require user
confirmation, block new writes, create a backup/checkpoint where practical, and
run the migration with clear failure recovery.

v0.2 should include a basic manual backup/restore path for the local data
library. Migration and repair flows should create checkpoints where practical,
but backup/restore is also a normal user-visible safety tool.

The technical policy should account for different update cadences:

- document the supported Fricon Desktop, CLI, Python SDK, local-service, and
  data-library compatibility envelope
- protect recorded data and provide explicit migrations even when v0.x APIs or
  protocols break
- gate newer APIs behind explicit capability negotiation where practical so old
  pinned scripts fail clearly instead of seeing partially supported behavior
- let newer clients discover older services and fail clearly when a required
  capability is missing
- perform startup compatibility checks before mutating a data library
- show clear recovery guidance when a client, service, or data library is
  incompatible
- defer polished auto-update UX until the replacement workflow is proven

Do not promise long-term third-party protocol stability in v0.2. Do define the
internal Fricon client/service compatibility contract early enough that locked
Python measurement environments fail before writes and tell the user what to
update when routine desktop or service updates make them incompatible.

## Client/Server Compatibility Boundary

The local service protocol is a technical contract, but the compatibility
experience is product-visible.

v0.2 should converge public Fricon clients on one browser-capable service
contract:

```text
Fricon Desktop frontend
  -> HTTP control/metadata API
  -> WebSocket or SSE live events
  -> binary dataset payload endpoints

Python SDK
  -> same HTTP control/metadata API
  -> same binary dataset write/read endpoints
  -> optional WebSocket or SSE subscriptions

CLI
  -> same service API
```

The current gRPC transport is implementation background, not the preferred
durable v0.2 public protocol. Fricon currently uses mostly unary gRPC calls plus
one client-streaming dataset create path. Dataset writes already require manual
Arrow IPC payload chunking to avoid large per-message payloads, so gRPC is not
removing the main transfer complexity. Keeping Python on gRPC while moving the
GUI to HTTP/WebSocket would create two public protocol stacks and a harder
compatibility matrix.

Prefer a service contract shaped around:

- JSON HTTP for metadata, control, compatibility negotiation, update status,
  and ordinary mutations
- WebSocket or Server-Sent Events for live measurement, dataset, and service
  status events
- explicit dataset write sessions with create, append, finish, and abort
  operations
- binary Arrow IPC or Arrow-compatible chunk payloads for dataset reads and
  writes
- server-side paging, summaries, and downsampling for UI reads instead of
  row-by-row JSON transfer

gRPC may remain temporarily during migration or reappear later as an internal
or high-performance transport, but v0.2 should not make it the public Python SDK
contract unless an ADR proves the benefit outweighs the extra browser and
compatibility cost.

The service API ADR should decide:

- how clients discover the running local service
- how Fricon Desktop, CLI, and Python SDK launch or request launch of the
  bundled local service when it is not running
- how Fricon Desktop, CLI, and Python SDK report their service API and client
  capability versions
- how the service reports its protocol and data-library format version
- how capability negotiation distinguishes read, write, measurement creation,
  export, and migration operations
- the v0.x compatibility policy for older Python SDKs used by locked lab
  scripts and notebooks: durable data and clear failure/migration guidance, not
  strict API stability
- how the service reports active blockers, draining state, and update-safe
  status
- which operations are read-only-safe during a compatibility mismatch
- how migration or upgrade prompts are surfaced

The exact HTTP route shape, event transport, binary payload format, and
compatibility envelope should be an ADR before durable implementation. The
important behavior is that incompatible clients fail before writes, tell the
user what to update, and never leave partially written measurement data.

Do not choose a Tauri-only IPC path for core business data. Do not preserve gRPC
as a separate public Python SDK path just because it is the current transport.

## Fricon Desktop And Future Web UI

Build the core UI as a web application that can be packaged as desktop.

Preferred direction:

- make Fricon Desktop the primary v0.2 GUI
- make the measurement console the first screen
- support detachable measurement, plot, or data windows for watching multiple
  runs without creating multiple independent full app instances
- highlight newly started Python measurements in the console and live list
  without stealing focus or opening windows automatically
- treat live views as operational monitoring surfaces, not publication plotting
  tools
- provide default shortcuts for today, live/active, active sample/session,
  favorites, partial/failed, trash, and text search
- bundle the frontend assets in the Tauri desktop app for normal local use
- use the local service API for core data access instead of routing dataset
  queries through Tauri commands
- avoid Tauri-only assumptions in measurement, dataset, sample/session, notes,
  tags, and live-chart feature code
- keep desktop-specific file dialogs, offline bundle file association, launch
  behavior, updater, diagnostics, and local service management behind shell
  adapters
- keep the frontend browser-capable enough that a future service-served remote
  viewer or PWA-like mode can reuse the core UI

This keeps the v0.2 user experience local and desktop-friendly while leaving a
clean path to remote monitoring and browser access. It does not require v0.2 to
ship remote mode or a browser/PWA distribution.

## Local-First Service Model

The default deployment is local-first:

```text
Fricon Desktop / Python SDK / CLI
  -> local Fricon service
  -> one Fricon data library
```

The service owns coordinated access to storage, live events, dataset writes,
run records, parameter snapshots, and future automation.

The local service is the authoritative data backend. The desktop app may launch
or supervise it, but core reads and writes should go through the service API so
future CLI, Python SDK, desktop remote mode, and browser clients share one
compatibility boundary.

The data library should be the user-facing concept. `WorkspaceRoot` or similar
names may remain internal implementation details, but public docs and UI should
avoid encouraging many long-lived workspaces.

Each data library should get a generated UUID and user-editable display name at
creation time. Exported measurements and audit summaries should include this
identity, plus an optional source computer label, so researchers can tell where
portable data came from.

The service should coordinate multiple concurrent local measurement writers.
Each active writer belongs to an isolated measurement record and reports live
status to the console.

Measurement list DTOs should provide a human-first display identity: name or
title, start time, sample/session label when available, and stable record ID as
secondary technical identity.

Live views, preview transforms, export preparation, and future analysis hooks
should be noncritical consumers of the acquisition stream. They must not block
or fail dataset writes; use async event delivery, bounded queues, backpressure,
or dropped preview updates where needed.

## Future Remote Access

Plan for remote access early, but do not make v0.2 ship remote mode or become a
multi-user hosted system.

Future remote access should support:

- read-only viewing of live measurement progress from another machine
- read-only browsing of historical runs and datasets
- notebook analysis from a different workstation
- monitoring scheduled calibration

Remote access should not require:

- teams
- roles
- permissions matrix
- account management
- distributed database behavior

Do not support direct multi-machine access by opening a database-backed Fricon
data library from a shared folder. A lab machine should own the data library
through the Fricon service, and other machines should connect as clients when
remote access is implemented.

The first remote phase should be strict read-only LAN monitoring, browsing, and
export. Remote annotations, remote acquisition writes, collaboration semantics,
and multi-user administration remain later scope.

## Authentication And Actor Boundary

Do not build full multi-user authorization now. Do create an early connection
and actor boundary so remote access and auditability are not retrofitted later.

Recommended v0.2 policy:

- local access uses a generated local token, not unauthenticated open loopback
  writes
- every mutating operation can record an actor, including an optional local
  operator profile when configured
- operator/profile selection should be a machine or session default rather than
  a per-measurement prompt
- the first authorization model is single-owner: authenticated clients can act
  as the library owner
- future remote access must require token or pairing
- roles and fine-grained permissions remain future scope

Suggested internal actor shape:

```text
Actor
  kind: local_user | api_token | service | automation
  display_name
```

Use actors for audit/provenance before using them for permissions.

## Storage Model Direction

Storage should support one data library with explicit domain records:

```text
DataLibrary
  Sample
  SampleSession
  MeasurementRun
  Artifact
  DatasetArtifact
  AttachmentArtifact
  AnalysisResult
  ParameterSnapshot
  ParameterProposal
  CodeSourceSummary
  DeviceIdentity
  Event/AuditRecord
```

Dataset payloads can continue to use Arrow chunk concepts where appropriate.
The catalog and provenance model should be redesigned around the broader data
library, not around dataset-only ownership. `DatasetArtifact` should be the
first concrete artifact type, but storage should reserve the general
`Artifact` boundary for reports, figures, logs, attachments, code-source
summaries, waveform or configuration files, and future device snapshots.

For v0.2, table-shaped dataset artifacts and light measurement attachments are
the concrete scope. Dataset contents should be appendable while their writer is
active and immutable after finish; fixes should create derived artifacts or
correction events.

Measurement records should reserve a passive code-source summary so run
history can explain which lab code source, local checkout, entry point, and
environment likely produced the data. This summary should be linked to the
measurement, not stored as dataset-local metadata.

Measurement notes and markers should be stored as timestamped events in the
measurement event timeline, beside lifecycle and system events, rather than as
only one mutable text field.

Datasets intended for live or historical plotting should require explicit scan
schema at creation time. The schema should carry the acquisition code's own
knowledge of setpoint/independent columns, measured/dependent columns,
fixed/config values, monitor/readback values, axis order or shape when known,
units/labels, limits when available, and enough role metadata for slicing and
display. Scratch or unplotted lower-level tables may use a generated guessed
schema, but guessed schema should not be the primary path for measurement data.

## API Model Direction

The ergonomic measurement path should be measurement-scoped:

```python
# Notebook prelude, exact API unsettled.
lib = fricon.library()
lib.use_context(sample="qpu-017", session="cooldown-2026-05")

with lib.measurement("rabi q3") as meas:
    rabi = meas.dataset(
        "rabi",
        scan={"independent": "amp", "dependent": "signal"},
    )
    for amp in amps:
        rabi.write(amp=amp, signal=measure(amp))
```

Sample/session context should be a resolved default, not an implicit hidden
global. The measurement record should store the resolved sample/session IDs
when active context is used. If no context is selected, the run remains valid
and the UI should make attach-later correction explicit.

The library handle should be cheap to keep in a notebook variable and reuse
across cells. It should not require context-manager cleanup in normal examples.

Lower-level dataset creation remains useful:

```python
lib = fricon.library()

with lib.dataset("scratch") as ds:
    ds.write(x=1.0, y=2.0)
```

Lower-level datasets are unassigned unless explicitly linked to a producer
record.

The active-context idea should be available from both Python prelude code and
Fricon Desktop because existing lab notebooks often set a save path near the
top of the file.

Active context should remain visible, but Fricon should not depend on
overconfident stale-context heuristics. The API and UI should make it cheap to
bulk-correct sample/session context on recent measurements with event history.

Measurement creation should be explicit but short. Do not hide automatic
measurement creation behind low-level dataset writes in the normal SDK path.
If a script crashes, the partial measurement remains visible and a rerun
creates a new linked measurement by default; appending to the old measurement
requires explicit resume intent and compatibility checks.

The v0.2 measurement API should allow optional setup/method labels and basic
clock or timing-source metadata. These are run context fields, not a device
management framework.

## Portable Export Direction

Exports should be measurement-centered and portable.

The first export API should let users export a measurement and read the
result on another computer without creating or importing into a local data
library:

```python
lib = fricon.library()
meas = lib.measurements.get("meas_123")
bundle_path = meas.export("rabi-q3.fricon-export")

bundle = fricon.open_export(bundle_path)
rabi = bundle.measurement("rabi q3").dataset("rabi").to_pyarrow()
```

The exact syntax is unsettled. The contract is:

- export bundles are read-only portable artifacts
- Python can open bundles directly
- Fricon Desktop can open bundles in a dedicated export viewer mode without
  requiring a running local service
- importing into another data library is optional and separate
- source data library UUID, display name, optional source computer label,
  export UUID, format version, original record IDs, checksums, and Fricon
  version travel in the bundle
- common tabular files such as CSV or Parquet should be included when practical
- a simple human-readable manifest or index preview should be included
- measurement metadata, sample/session context, produced datasets, selected
  non-table artifacts, parameter snapshot or legacy metadata, code/environment
  summary, setup/method labels, basic timing metadata, notes, tags, lifecycle
  flags, and provenance summaries travel with the exported measurement
- Python loader snippets should treat the export as an analysis package for
  laptop or HPC work, not as an opaque archive that must be imported first
- sensitive provenance such as full paths, detailed dirty Git state, full
  environment summaries, source computer label, and extensive sample metadata
  should be opt-in or explicitly previewed before export

## Minimal Device Boundary

v0.2 should reserve a typed device boundary without implementing a broad driver
framework or managed device communication.

The boundary should include:

- device identity and aliases
- declared capabilities, including readable state, writable state, units,
  limits, and safety hints
- an adapter boundary that can later wrap LabRAD, direct Python drivers, VISA,
  serial, vendor SDKs, or dummy devices
- desired state, apply plan, readback, and partial failure summaries
- resource keys for future leases

This keeps a path toward complete LabRAD replacement without forcing v0.2 to
ship a general-purpose hardware framework. Existing LabRAD scripts should
migrate through the Fricon SDK; do not emulate Data Vault/Grapher behavior in
v0.2.

## Managed Measurement Plan Direction

Simple interactive measurements should remain imperative Python. Fricon should
not require declarative managed plans for exploratory measurement.

For repeated, retryable, or automation-heavy work, v0.2 should leave room for an
optional managed measurement plan model:

```text
parameter snapshot
  + run-local inputs
  + scan point
  -> desired device state
  -> device apply plan
  -> measurement step
  -> dataset writes
  -> post-processing hooks
```

This is similar in spirit to a declarative UI tree: the user describes the
desired state for a point in the scan, and the managed runner reconciles that
desired state with devices, datasets, and provenance.

The managed plan model should enable:

- previewing the device state before execution
- diffing desired device state against current or last-applied state
- dummy-device and dry-run execution
- explicit scan point identity for resume and retry
- post-processing hooks after a point, sweep, or dataset completes
- safer automatic calibration workflows
- readback verification and partial failure reporting

The API should be optional. Existing imperative code can continue to create
interactive runs and write datasets directly. Advanced managed features such as
resume, retry, dry-run, resource leases, automatic calibration, and device
readback may require the declarative plan or explicit advanced API hooks.

Example shape for discussion, not settled API:

```python
@fricon.measurement_template
def rabi(ctx, params, scan):
    amp = scan.axis("amp")

    signal = ctx.dataset("rabi")

    for point in scan.points(amp=linspace(0.0, 1.0, 101)):
        desired = {
            "awg.q3.x90_amp": params["q3/x90/base_amp"] * point.amp,
            "vna.if_bandwidth": params["devices/vna/if_bandwidth"],
        }
        with ctx.point(point, desired_device_state=desired) as p:
            p.apply_devices()
            signal.write(amp=point.amp, response=measure_response())
```

The important contract is not this exact syntax. The important contract is that
Fricon can derive an inspectable desired device state from parameter snapshots
and scan points before executing hardware mutations.

Acquisition should remain Python-first. Do not plan a Labber-like visual sweep
builder as a product goal before explicit scan schema and optional managed-plan
previews prove that users need a visual authoring layer.

## Code And Environment Provenance

v0.2 should avoid encouraging copied code directories.

Start with optional passive summaries:

- user-provided code label and script path or module entry point when available
- source kind and label when known, such as Git repository, read-only network
  mirror, release package, or local folder
- optional Git remote, tag, commit, dirty state, untracked-change summary, or
  file hash summary when available
- Python version
- Fricon version
- lock-file or environment summary when practical

Do not make v0.2 a full Git client or environment manager. Reserve that for a
later explicit reproducibility feature.

## Measurement Code Source Direction

The measurement-code source is separate from the Fricon data library. Each
acquisition computer should keep:

- its own local data library
- a local checkout, installed package, or local folder for measurement code
- its own Python environment or environment lock
- local setup overrides for device addresses, ports, paths, and secrets

Shared infrastructure can still help:

- a Git repository for maintainers and advanced users
- a read-only network mirror for labs that already rely on shared storage
- a package cache or release bundle for locked-down Windows lab PCs
- shared setup profiles, measurement templates, scan-schema helpers, plot
  presets, export recipes, driver/helper modules, and future calibration
  workflow definitions

The service should record passive code-source facts when a measurement starts,
but should not mutate source code in the v0.2 replacement slice. Future setup
and update tooling can wrap existing Git or package operations through
experimenter-facing actions:

- install approved measurement code
- update to an approved release or tag
- show what changed
- check the Python environment
- warn about dirty or unknown local code before measurement
- export local changes for maintainer review

Do not make network storage the active shared data library. Do not make a
shared editable network directory the primary way to run measurement code on
multiple computers. Network storage can be a mirror, package cache, export
target, or backup target.

## Parameter And Calibration Direction

Large parameter sets should eventually be explicit, versioned, and reviewable.
v0.2 can start with optional flexible parameter snapshots attached to
measurements, not a full global parameter registry:

```text
mutable parameter ref/profile
  -> resolved immutable snapshot before a run
  -> run-local overrides recorded on the run
  -> analysis result proposes patch
  -> validation
  -> commit new snapshot
  -> optional ref promotion
```

Automatic calibration should be built as workflow over explicit records:

```text
CalibrationWorkflowRun
  -> managed measurement plan/run
  -> measured dataset
  -> analysis result
  -> parameter proposal
  -> validation
  -> approval or policy gate
  -> ref update
```

No calibration workflow should silently mutate an important profile during data
collection.

Analysis execution and automatic calibration are not v0.2 user-facing
workflows. The model should reserve room for them, but v0.2 analysis remains
external through Python/export.

## ADRs Needed Before Implementation

Likely ADRs:

- v0.2 product and data-library repositioning
- supported distribution surfaces and install/update policy
- data library versus workspace public model
- sample and sample-session identity
- active sample/session context and attach-later correction policy
- general Artifact versus DatasetArtifact boundary
- dataset artifact and provenance model
- plotted dataset scan schema contract and guessed-schema fallback
- authentication/actor boundary for local access and future remote access
- client/server protocol compatibility, version negotiation, and
  fail-before-write diagnostics for incompatible clients
- measurement-code source/package model, new-computer setup, and the boundary
  between Fricon-managed actions and ordinary Git/environment tools
- public naming policy for Measurement versus Experiment
- Python SDK surface and measurement-scoped dataset writer lifecycle
- minimal device adapter and capability boundary for future LabRAD replacement
- optional managed measurement plan and desired-device-state boundary
- storage compatibility and migration policy for pre-v0.2 workspaces
- Fricon Desktop shell boundary, local service ownership, and future
  remote/browser UI access
- service sidecar packaging, optional login startup, update-safe/draining
  lifecycle, data-library migration gating, and backup/restore policy

## First Engineering Slice

Before broad implementation, build a narrow vertical slice that proves the new
foundation:

1. Create/open one data library.
2. Create or select sample/session context when known, or explicitly leave it
   unset.
3. Set active sample/session context from Python prelude or Fricon Desktop when
   appropriate.
4. Start an explicit measurement from Python, including headless script use.
5. Write one or more dataset artifacts through measurement-scoped handles,
   including explicit scan schema for datasets intended for plotting.
6. Browse the run and datasets in the Fricon Desktop measurement console.
7. Reopen a dataset from Python by stable ID.
8. Attach or correct sample/session context after the run when needed.
9. Record actor/operator label, passive code-source summary, run note,
   lifecycle flags, and sample/session links.
10. Export a read-only measurement bundle with common tabular files, a simple
    manifest/index preview, and direct Python/Desktop offline-viewer access.
11. Exercise backup/restore and trash/recover as user-visible safety paths.

This slice should intentionally break old workspace/dataset assumptions where
they conflict with the v0.2 model.
