# Product Vision

## Status

Draft v0.2+ product baseline.

## Thesis

Fricon v0.2+ is a local lab data library and automation foundation for
scientific measurement work.

The first product goal is practical: replace a simple LabRAD Grapher/Data Vault
style logger for new measurements without building a compatibility layer for
old storage.

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

## First v0.2 Slice

The first shipped slice should include:

- one local data library per normal lab computer
- explicit measurements
- optional sample and sample-session context
- table-shaped dataset artifacts with explicit scan schema for plotted data
- nonblocking live table and chart inspection
- measurement lifecycle, notes, events, favorites/pins, trash/recover
- light attachments
- optional flexible parameter snapshot
- honest code provenance summary
- Python reopen snippets through public APIs
- measurement-centered portable export
- backup/restore and migration checkpoints
- guided setup and compatibility diagnostics

## Later Layers

These are important but not first-slice commitments:

- read-only LAN monitoring
- richer sample fields and 2D sample maps
- parameter profiles, proposals, and calibration promotion
- managed code snapshots and managed script execution
- managed measurement plans
- analysis and calibration activity records
- device identity and communication
- AI-assisted reviewable automation

## Non-Goals For v0.2

- hosted SaaS
- account/team administration
- multi-user permissions
- distributed database semantics
- direct shared-folder access to one editable data library
- LabRAD Data Vault/Grapher compatibility server
- built-in legacy Data Vault import or browser
- broad device-driver framework
- generic workflow DAG engine
- visual sweep builder as the primary acquisition model
- automatic notebook state capture
- AI actions that mutate data-library state without explicit review and audit
