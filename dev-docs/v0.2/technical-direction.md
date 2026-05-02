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
fricon desktop GUI
fricon CLI
fricon Python SDK
```

They should operate against one local data-library service and storage model:

```text
Fricon Data Library
  local service / daemon
  storage
  API
  desktop GUI or browser UI
  CLI
  Python SDK
```

The Python SDK remains a first-class surface for measurement scripts and
analysis notebooks. The desktop GUI is the primary browsing, inspection,
sample, and calibration-monitoring surface. The CLI should handle setup,
library management, import/export, diagnostics, and service control.

## Installation And Update Experience

Installation and update are user-facing v0.2 concerns, even though their
implementation details are technical.

The product-level requirement is:

```text
Fricon distribution
  -> desktop GUI and local service can update together
  -> fricon CLI follows the installed service contract
  -> Python SDK may be pinned in lab environments
  -> data-library format upgrades are explicit
```

Users should be able to install Fricon, create or open a data library, launch
the desktop UI, and connect from Python without understanding service internals.

The technical policy should account for different update cadences:

- document the supported desktop GUI, CLI, Python SDK, local-service, and
  data-library compatibility envelope
- keep the core v0.x Python SDK measurement-write and dataset-read APIs
  compatible with later v0.x local services after v0.2 lands
- gate newer APIs behind explicit capability negotiation so old pinned scripts
  can continue to run without seeing partially supported behavior
- let newer clients discover older services and fail clearly when a required
  capability is missing
- perform startup compatibility checks before mutating a data library
- show clear recovery guidance when a client, service, or data library is
  incompatible
- defer polished auto-update UX until the replacement workflow is proven

Do not promise long-term third-party protocol stability in v0.2. Do define the
internal Fricon client/service compatibility contract early enough that locked
Python measurement environments are not broken by ordinary desktop or service
updates within v0.x.

## Client/Server Compatibility Boundary

The local service protocol is a technical contract, but the compatibility
experience is product-visible.

v0.2 should decide:

- how clients discover the running local service
- how the desktop GUI, CLI, and Python SDK report their client protocol version
- how the service reports its protocol and data-library format version
- how capability negotiation distinguishes read, write, measurement creation,
  export, and migration operations
- the core v0.x compatibility promise for older Python SDKs used by locked lab
  scripts and notebooks
- which operations are read-only-safe during a compatibility mismatch
- how migration or upgrade prompts are surfaced

The exact IPC, HTTP, websocket, or gRPC shape should be an ADR before durable
implementation. The important behavior is that v0.2-era Python SDKs keep
working for ordinary measurement recording and dataset reads against later v0.x
services, while incompatible clients fail before writes and tell the user what
to update.

## Desktop GUI And Web UI

Build the UI as a web application that can be packaged as desktop.

Preferred direction:

- keep a Tauri desktop package for local use
- make the frontend able to run against HTTP/websocket APIs when served in a
  browser
- avoid Tauri-only assumptions in feature code where practical
- keep desktop-specific file dialogs, launch behavior, and local service
  management behind adapters

This keeps the normal user experience local and desktop-friendly while leaving
a clean path to remote monitoring and browser access.

## Local-First Service Model

The default deployment is local-first:

```text
desktop GUI / Python SDK / CLI
  -> local Fricon service
  -> one Fricon data library
```

The service owns coordinated access to storage, live events, dataset writes,
run records, parameter snapshots, and future automation.

The data library should be the user-facing concept. `WorkspaceRoot` or similar
names may remain internal implementation details, but public docs and UI should
avoid encouraging many long-lived workspaces.

Each data library should get a generated UUID and user-editable display name at
creation time. Exported measurements and audit summaries should include this
identity, plus an optional source computer label, so researchers can tell where
portable data came from.

## Remote Access

Plan for remote access early, but do not make v0.2 a multi-user hosted system.

Remote access should support:

- viewing live measurement progress from another machine
- browsing historical runs and datasets
- notebook analysis from a different workstation
- monitoring scheduled calibration

Remote access should not require:

- teams
- roles
- permissions matrix
- account management
- distributed database behavior

## Authentication And Actor Boundary

Do not build full multi-user authorization now. Do create an early connection
and actor boundary so remote access and auditability are not retrofitted later.

Recommended v0.2 policy:

- local loopback access may use implicit local trust or a generated local token
- remote access must require token or pairing
- every mutating operation can record an actor
- the first authorization model is single-owner: authenticated clients can act
  as the library owner
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
  CodeSnapshot
  DeviceIdentity
  Event/AuditRecord
```

Dataset payloads can continue to use Arrow chunk concepts where appropriate.
The catalog and provenance model should be redesigned around the broader data
library, not around dataset-only ownership. `DatasetArtifact` should be the
first concrete artifact type, but storage should reserve the general
`Artifact` boundary for reports, figures, logs, attachments, code summaries,
waveform or configuration files, and future device snapshots.

## API Model Direction

The ergonomic measurement path should be measurement-scoped:

```python
# Notebook prelude, exact API unsettled.
lib = fricon.library()
lib.use_context(sample="qpu-017", session="cooldown-2026-05")

with lib.measurement("rabi q3") as meas:
    rabi = meas.dataset("rabi")
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
the desktop UI because existing lab notebooks often set a save path near the
top of the file.

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
- the desktop GUI can open bundles in a dedicated export viewer mode
- importing into another data library is optional and separate
- source data library UUID, display name, source computer label, export UUID,
  format version, original record IDs, checksums, and Fricon version travel in
  the bundle
- measurement metadata, sample/session context, produced datasets, selected
  non-table artifacts, parameter snapshot or legacy metadata, code/environment
  summary, notes, tags, quality state, and provenance summaries travel with
  the exported measurement

## Minimal Device Boundary

v0.2 should reserve a typed device boundary without implementing a broad driver
framework.

The boundary should include:

- device identity and aliases
- declared capabilities, including readable state, writable state, units,
  limits, and safety hints
- an adapter boundary that can later wrap LabRAD, direct Python drivers, VISA,
  serial, vendor SDKs, or dummy devices
- desired state, apply plan, readback, and partial failure summaries
- resource keys for future leases

This keeps a path toward complete LabRAD replacement without forcing v0.2 to
ship a general-purpose hardware framework. If LabRAD is needed during
migration, it should sit behind an adapter boundary rather than remain the
conceptual model.

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

## Code And Environment Provenance

v0.2 should avoid encouraging copied code directories.

Start with passive summaries:

- script path or module entry point
- Git commit, dirty state, or file hash summary when available
- Python version
- Fricon version
- lock-file or environment summary when practical

Do not make v0.2 a full Git client or environment manager. Reserve that for a
later explicit reproducibility feature.

## Parameter And Calibration Direction

Large parameter sets should be explicit, versioned, and reviewable:

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

## ADRs Needed Before Implementation

Likely ADRs:

- v0.2 product and data-library repositioning
- supported distribution surfaces and install/update policy
- data library versus workspace public model
- sample and sample-session identity
- active sample/session context and attach-later correction policy
- general Artifact versus DatasetArtifact boundary
- dataset artifact and provenance model
- authentication/actor boundary for local and remote access
- client/server protocol compatibility, version negotiation, and core v0.x
  Python SDK compatibility policy
- public naming policy for Measurement versus Experiment
- Python SDK surface and measurement-scoped dataset writer lifecycle
- minimal device adapter and capability boundary for future LabRAD replacement
- optional managed measurement plan and desired-device-state boundary
- storage compatibility and migration policy for pre-v0.2 workspaces
- desktop web architecture and remote UI access

## First Engineering Slice

Before broad implementation, build a narrow vertical slice that proves the new
foundation:

1. Create/open one data library.
2. Create or select sample/session context when known, or explicitly leave it
   unset.
3. Set active sample/session context from Python prelude or desktop UI when
   appropriate.
4. Start an interactive measurement from Python.
5. Write one or more dataset artifacts through measurement-scoped handles.
6. Browse the run and datasets in the desktop/web UI.
7. Reopen a dataset from Python by stable ID.
8. Attach or correct sample/session context after the run when needed.
9. Record actor, code summary, run note, quality, and sample/session links.

This slice should intentionally break old workspace/dataset assumptions where
they conflict with the v0.2 model.
