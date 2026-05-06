# Product Vision

## Status

Accepted clean-reset product baseline.

## Thesis

Fricon is a local lab data library and automation foundation for scientific
measurement work.

## Planning Language

This document uses MVP, post-MVP, and ADR-gated as product priority labels.
They are not semantic-version labels. Compatible improvements can still ship on
the same compatible release line when the storage, API, and compatibility
policies allow it.

## Long-Term Motivation

Physical measurement work is hard to make reliable when data, parameters,
measurement code, setup state, notes, and later analysis live in separate tools
or informal files. Existing frameworks can help users collect data, but they
often leave the broader experiment record to conventions that are difficult to
inspect, compare, migrate, or automate.

Fricon should give experimenters one local-first product model for defining,
running, inspecting, explaining, and reusing measurement work. The long-term
aim is not only to store results, but to make the relationship between a
measurement, its datasets, its Python code, its context, and its later
interpretation explicit enough for humans and future automation to trust.

The product should stay close to how experimentalists already work: Python
scripts and notebooks remain first-class, local lab computers remain useful
without a server account model, and higher-provenance workflows grow from the
same core experience instead of becoming a separate system.

## MVP Goal

The MVP goal is practical: replace the simple LabRAD Data Vault/Grapher loop
for new measurements.

Success means a user can start new measurement work in Fricon, write data from
Python, watch it live, recover partial results, reopen it later, and export it
without depending on old storage paths. The MVP does not need to import old
history or emulate LabRAD; old LabRAD, QCoDeS, Labber, or folder-based history
can remain where it is while new work moves to Fricon.

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

At the vision level, the SDK should feel like ordinary Python with low
ceremony: a visible notebook context, natural interactive unmanaged runs,
importable managed-run entry points for higher provenance, concise doAnd-style
helpers for routine scans, and public reopen/export APIs for later analysis.

Detailed SDK usage guidance lives in `product/python-sdk-ux.md`. Old planning
snippets under `dev-docs/` should be read as non-binding UX sketches unless an
ADR accepts exact API syntax.

## MVP Scope

To meet the MVP goal, the MVP should include:

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
- light contextual summaries for parameters, code provenance, setup,
  environment, and unmanaged procedure context
- Python reopen snippets through public APIs
- a near-term measurement export spec for portable bundles and common analysis
  formats
- backup/restore and migration checkpoints
- coherent install/update compatibility and guided setup diagnostics

## Post-MVP Direction

These directions matter to the long-term product, but are not MVP
commitments:

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

## Non-Goals For MVP

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
