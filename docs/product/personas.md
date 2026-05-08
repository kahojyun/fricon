# Personas

## Status

Accepted.

## Purpose

These personas are lightweight product-role archetypes for Fricon planning.
They are not fictional biographies, job titles, a permission model, or a
replacement for user stories.

Use this document to decide whose goal, risk, or decision a product slice
serves. Put concrete capabilities, success criteria, and edge cases in user
stories, future requirements, or domain documents.

## Grounding

These archetypes were refined against a representative local lab workspace,
which shows current practice built around Windows lab folders, Jupyter
notebooks, LabRAD Data Vault, mutable `parameters.json`/`registry.json` files,
wiring spreadsheets, generated sidecars, calibration scripts, hardware
bring-up helpers, backups, and report artifacts.

They are responsibility views, not fixed people. One person may move between
roles during a single day.

## Writing Rules

- Keep each persona short, research-backed, and easy to remember.
- Describe stable goals, behavior patterns, context, and decision pressure.
- Do not repeat feature requirements that belong in user stories.
- Use story modifiers for temporary contexts such as first-time use, offline
  operation, locked-down machines, interrupted runs, or migration work.
- Add a new persona only when research shows a durable product responsibility
  that the existing roles cannot explain.

## Story Routing

Choose the persona that owns the story's main outcome:

- P-001 Experimentalist: ordinary measurement operation and recovery.
- P-002 Fricon Technical Maintainer: Fricon deployment, local runtime health,
  diagnostics, migration support, and technical readiness.
- P-003 Analyst: reopening, analysis, reports, derived artifacts, downstream
  handoff, and interpretation provenance.
- P-004 Calibration and Parameter Steward: calibration evidence, parameter
  state, fitted values, effective configuration, and rollback decisions.
- P-005 Measurement Stack Author: reusable measurement code, SDK shape, scan
  helpers, runner integration, plotting utilities, and report/export recipes.

When several roles participate, use the accountable role as the story's
primary persona and mention supporting roles in notes or acceptance criteria.

If a story asks Fricon to mutate durable lab state, the accountable role is
never only P-001. Route the decision to the state owner: P-004 for parameter
or calibration proposals, P-005 for managed-routine or measurement-code
proposal shape, and P-003 for durable analysis or interpretation records.
P-002 contributes technical guardrails when runtime, update, library, or
environment safety matters.

## P-001 Experimentalist

Researchers with entry-level Python ability who run measurement scripts or
notebooks on lab computers. They may use routines written by a more advanced
lab member and may only know enough Python to change a scan range, select
qubits, rerun a cell, or add a note.

Goals:

- start, watch, annotate, recover, reopen, and export measurement work
- understand the active sample/session, measurement name, scan shape, units,
  and selected local context without learning Fricon internals
- preserve useful partial results when scripts interrupt or long saves fail

Context and pressure:

- often works on Windows, offline, locked-down, or slow-to-update lab machines
- may depend on Jupyter, Conda or `uv`, LabRAD/Data Vault services, copied
  folders, dated backups, and mutable local configuration during migration
- needs Fricon to make ordinary measurement work safer without forcing managed
  execution first

Common switches:

- becomes P-004 when deciding whether calibration results should change working
  parameter state
- depends on P-005 for reusable routines and P-002 when setup or update
  problems block measurement work

## P-002 Fricon Technical Maintainer

The person who develops, deploys, updates, diagnoses, and supports Fricon for a
lab's local computers and data libraries.

Goals:

- keep Fricon installation, local runtime components, data libraries, Python
  environments, and update paths technically usable
- make setup, compatibility, migration, and support problems diagnosable
  before users read raw logs
- help labs adopt Fricon gradually while old LabRAD/Data Vault, folder, and
  parameter-file history remains in place

Context and pressure:

- supports machines that may be offline, locked down, slow to update, or
  pinned to lab-specific Python environments
- must handle sensitive local paths, machine names, IP addresses, environment
  details, and setup files carefully
- contributes technical readiness checks when a proposal depends on runtime,
  update timing, data-library compatibility, or environment state

Common switches:

- works with P-005 when shared code sources or package environments become
  lab-maintained assets
- relies on P-004 or P-003 for scientific acceptance of calibration,
  parameter, analysis, or interpretation outcomes

## P-003 Analyst

The person who reopens completed measurements for notebooks, reports, or HPC
analysis. This role often produces secondary artifacts such as fit results,
plots, `.npy` or `.json` derived data, spreadsheets, and slide decks.

Goals:

- reopen completed, interrupted, or exported measurements through stable IDs
  and APIs
- preserve links from derived artifacts back to source measurements,
  parameters, code/procedure context, and manual judgment
- keep mapping, readout-classification, shot-group, detector, or observable
  context aligned for advanced analysis workflows

Context and pressure:

- often works away from the acquisition computer or after the run has finished
- needs privacy-aware handoff when exports contain local paths, setup details,
  code summaries, environment data, or sample context
- must distinguish recorded measurement facts from notebook-local analysis
  state and manually edited outputs

Common switches:

- consumes measurements produced by P-001
- may provide evidence that P-004 uses for parameter or calibration decisions
- uses report/export recipes maintained by P-005

## P-004 Calibration and Parameter Steward

A lab member who tunes readout, pulse, coupler, crosstalk, demodulation,
feedback, hardware bring-up, or related parameters and decides whether fitted
values should become the next working local configuration.

Goals:

- decide which effective configuration or prior-good state calibration work
  depends on
- evaluate calibration evidence, fit quality, health gates, retry/pause needs,
  and manual inspection flags
- mark calibration results as exploratory, accepted, rejected, superseded, or
  needing review, with rollback context where practical

Context and pressure:

- works with mutable JSON files, lock files, dated backups, copied setting
  folders, generated temporary files, wiring sheets, and local database helpers
- may run routines that update local parameters while also producing datasets,
  so mutation and measurement facts must stay distinguishable
- may depend on physical instrument addresses and external device state that
  Fricon can record before it can control

Common switches:

- may be the same person as P-001 during daily calibration work
- depends on P-005 for calibration routines and code provenance
- uses P-002 technical readiness checks when calibration proposals affect
  runtime, update, environment, or data-library compatibility

## P-005 Measurement Stack Author

An advanced Python user who writes or maintains reusable measurement scripts,
scan helpers, pulse-generation rules, runner integrations, plotting utilities,
or export/report recipes used by other lab members.

Goals:

- integrate Fricon recording into existing scripts, notebooks, Data
  Vault-style helpers, runners, and scan utilities
- declare scan axes, dependencies, trace shapes, repeated shots, validity
  masks, runner identifiers, and procedure summaries from code
- preserve honest code provenance without claiming Fricon managed execution
  guarantees it does not yet provide

Context and pressure:

- writes routines that other users may run without understanding every
  dependency
- needs migration ergonomics that do not require rewriting the whole
  measurement stack at once
- creates post-MVP pressure for approved code sources, managed entry points,
  templates, and reviewed updates

Common switches:

- enables P-001 by making scripts and helpers easier to run and record
- helps P-003 produce traceable analysis and report artifacts
- helps P-004 produce calibration evidence and parameter-change proposals
