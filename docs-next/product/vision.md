# Product Vision

## Status

Accepted v0.2+ product baseline.

## Thesis

Fricon v0.2+ is a local lab data library and automation foundation for
scientific measurement work.

## Long-Term Motivation

Fricon is motivated by a gap in existing physical measurement workflows:
dataset logging can be usable, but parameter management, measurement-code
management, environment setup, run provenance, calibration history, and later
automation often remain fragmented.

Building separate systems on top of existing measurement frameworks tends to
produce inconsistent user experience and brittle conventions: parameters hide
in JSON files, code history depends on copied folders or dirty Git checkouts,
runner facts hide in logs, and provenance is reconstructed after the fact.

Fricon's long-term goal is a unified local-first experience where measurement
recording, parameter snapshots, managed code sources, SDK runner capture,
dataset artifacts, setup state, analysis/calibration records, and reviewed
automation share one coherent product model.

Dataset artifacts and scan semantics are foundational, but they are not the
product endpoint. The v0.2 measurement/data-library slice exists to create a
stable base for the parameter system, measurement-code management, managed
runner, and calibration/automation layers.

The first product goal is practical: replace a simple LabRAD Grapher/Data Vault
style logger for new measurements without building a compatibility layer for
old storage.

The adoption promise is incremental: users should be able to start new
measurements in Fricon while old LabRAD, QCoDeS, Labber, or folder-based
history stays where it is. Fricon may record source aliases and legacy
references, but v0.2 should not require a historical migration before new data
collection can move forward.

## User Promise

Fricon should help a researcher answer:

- What measurement did I run?
- Which sample or session was active, if any?
- What datasets and attachments did it produce?
- Was the run finished, interrupted, failed, invalidated, or recovered?
- What notes, parameters, code provenance, and setup labels explain it?
- How do I inspect it live, reopen it from Python, export it, or recover it?

## Primary Mental Model

```text
I selected an active sample/session when it mattered.
I ran a measurement from Python.
It produced datasets.
Fricon helps me inspect, annotate, recover, reopen, and export them.
```

## Python SDK Experience

The Python SDK is a primary user experience, not only an implementation API.
Most experimentalists will define and run measurements through Python scripts
or notebooks, so SDK ergonomics are product requirements.

The SDK should:

- let users create explicit measurements with low ceremony
- support a visible notebook context for current library and optional lab
  context
- keep interactive unmanaged runs natural for exploratory Python
- make importable decorated managed runs possible later for higher-provenance
  work
- make doAnd-style scan helpers concise enough for routine scripts
- keep advanced raw schema available when helper APIs are too narrow
- return users to public Python read/reopen/export APIs instead of storage
  paths
- fail before mutation when Desktop, service, data library, CLI, or SDK versions
  are incompatible

Old planning snippets under `dev-docs/` should be read as UX sketches unless an
ADR accepts exact syntax. They express user requirements such as explicit
measurement creation, visible notebook context, interactive unmanaged runs,
importable decorated managed runs, doAnd-style helper ergonomics, and direct
Python reopen/export, not final API design.

## First v0.2 Slice

The first shipped slice should include:

- one local data library per normal lab computer
- explicit measurements
- optional sample and sample-session context
- dataset artifacts that remain directly searchable and openable, even though
  the Desktop home is measurement-first
- table-shaped scan and trace data with explicit scan schema for plotted data
- scan modes for regular grids, partial grids, irregular or adaptive points,
  repeated points, and fixed-shape or variable-length traces
- short scan-schema helpers for common 1D/2D/N-D scans and traces, plus a raw
  schema escape hatch for advanced cases
- nonblocking live table and chart inspection
- measurement lifecycle, notes, events, favorites/pins, trash/recover, and
  readable partial data semantics
- light attachments
- optional flexible parameter snapshot
- honest code provenance summary
- optional passive setup, device, and environment summary that describes
  context without controlling devices
- optional passive procedure summary that records unmanaged script, external
  runner, or declared plan context without implementing a runner
- Python reopen snippets through public APIs
- a near-term measurement export spec for portable bundles and common analysis
  formats
- backup/restore and migration checkpoints
- coherent install/update compatibility and guided setup diagnostics

## Later Layers

These are important but not first-slice commitments:

- parameter profiles, proposals, and calibration promotion
- parameter history, diffs, and proposal review
- managed measurement-code sources and approved-code update flows
- managed code snapshots and managed script execution
- SDK runner capture before scheduler/resource queues
- generated run history, compare, and operator handoff views
- read-only LAN monitoring
- richer sample fields and 2D sample maps
- managed measurement plans
- user-facing stream concepts inside dataset artifacts
- analysis and calibration activity records
- device identity and communication
- resumable managed execution with scan-point checkpoints
- AI-assisted reviewable automation

## Non-Goals For v0.2

- hosted SaaS
- account/team administration
- multi-user permissions
- distributed database semantics
- direct shared-folder access to one editable data library
- LabRAD Data Vault/Grapher compatibility server
- built-in legacy Data Vault import or browser
- requiring full old-history migration before adopting Fricon for new data
- broad device-driver framework
- generic workflow DAG engine
- visual sweep builder as the primary acquisition model
- automatic notebook state capture
- AI actions that mutate data-library state without explicit review and audit
