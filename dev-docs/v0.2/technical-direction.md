# Fricon v0.2 Technical Direction

## Status

Proposed v0.2 technical direction.

This is not an implementation plan. Use it to align architecture discussions,
ADRs, and early spike work.

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

The Python SDK remains a first-class surface for experiment scripts and
analysis notebooks. The desktop GUI is the primary browsing, inspection,
sample, and calibration-monitoring surface. The CLI should handle setup,
library management, import/export, diagnostics, and service control.

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

## Remote Access

Plan for remote access early, but do not make v0.2 a multi-user hosted system.

Remote access should support:

- viewing live experiment progress from another machine
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
  ExperimentRun
  DatasetArtifact
  AnalysisResult
  ParameterSnapshot
  ParameterProposal
  CodeSnapshot
  Event/AuditRecord
```

Dataset payloads can continue to use Arrow chunk concepts where appropriate.
The catalog and provenance model should be redesigned around the broader data
library, not around dataset-only ownership.

## API Model Direction

The ergonomic measurement path should be experiment-scoped:

```python
with fricon.library() as lib:
    sample = lib.samples.get("qpu-017")
    session = sample.session("cooldown-2026-05")

    with session.experiment("rabi q3") as run:
        rabi = run.dataset("rabi")
        for amp in amps:
            rabi.write(amp=amp, signal=measure(amp))
```

Lower-level dataset creation remains useful:

```python
with lib.dataset("scratch") as ds:
    ds.write(x=1.0, y=2.0)
```

Lower-level datasets are unassigned unless explicitly linked to a producer
record.

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
  -> managed experiment run
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
- data library versus workspace public model
- sample and sample-session identity
- dataset artifact and provenance model
- authentication/actor boundary for local and remote access
- Python SDK surface and experiment-scoped dataset writer lifecycle
- storage compatibility and migration policy for pre-v0.2 workspaces
- desktop web architecture and remote UI access

## First Engineering Slice

Before broad implementation, build a narrow vertical slice that proves the new
foundation:

1. Create/open one data library.
2. Create sample and sample session.
3. Start an interactive experiment from Python.
4. Write one or more dataset artifacts through experiment-scoped handles.
5. Browse the run and datasets in the desktop/web UI.
6. Reopen a dataset from Python by stable ID.
7. Record actor, code summary, run note, quality, and sample/session links.

This slice should intentionally break old workspace/dataset assumptions where
they conflict with the v0.2 model.
