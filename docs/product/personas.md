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

- P-001 Measurement Run Operator: ordinary measurement-run operation and
  recovery.
- P-002 Local Measurement System Maintainer: Fricon deployment, local runtime
  health, data libraries, Python/package environments, code-source setup,
  diagnostics, migration support, and technical readiness.
- P-003 Experimental Data Analyst: reopening, analysis, reports, derived
  artifacts, downstream handoff, and interpretation provenance.
- P-004 Effective Configuration Steward: effective configuration, calibration
  evidence, parameter state, fitted values, and rollback decisions.
- P-005 Measurement Method Author: measurement-method code, script and routine
  shape, SDK usage, scan helpers, runner integration, plotting utilities, and
  report/export recipes.

When several roles participate, use the accountable role as the story's
primary persona and mention supporting roles in notes or acceptance criteria.

If a story asks Fricon to mutate durable lab state, the accountable role is
never only P-001. Route the decision to the state owner: P-004 for parameter
or effective-configuration proposals, P-005 for managed-routine or
measurement-code proposal shape, and P-003 for durable analysis or
interpretation records. P-002 contributes technical guardrails when runtime,
update, library, or environment safety matters.

Physical setup, instrument, wiring, and device-state ownership is a distinct
future boundary. MVP stories record those facts as passive setup context or
run-bound local configuration. If later workflows review or mutate setup or
device state, route that decision to an explicit setup/device owner rather than
to P-002 by default; P-002 owns software-system readiness.

## P-001 Measurement Run Operator

The role active when a lab user runs a measurement script or notebook on a lab
computer. The person may be an experimentalist, student, or senior researcher,
but the product responsibility is operating and understanding a measurement
run, not every decision in the experiment.

Goals:

- start, watch, annotate, recover, and hand off measurement runs
- choose or adjust run-specific inputs such as scan ranges, selected targets,
  and context labels
- understand the active measurement name, scan shape, units, and selected local
  context without learning Fricon internals
- preserve useful partial results when scripts interrupt or long saves fail

Context and pressure:

- often works on Windows, offline, locked-down, or slow-to-update lab machines
- may depend on Jupyter, Conda or `uv`, LabRAD/Data Vault services, copied
  folders, dated backups, and mutable local configuration during migration
- needs Fricon to make ordinary measurement work safer without forcing managed
  execution first

Common switches:

- switches to P-004 when calibration evidence or fitted values are being judged
  for working effective configuration or parameter state
- switches to P-003 when the main work is analysis, reporting, or interpreting
  derived artifacts after acquisition
- switches to P-005 when changing measurement-method code, scan-helper
  semantics, procedure summaries, or dataset schema
- depends on P-002 when setup or update problems block measurement work

## P-002 Local Measurement System Maintainer

The role active when someone makes the local Fricon-backed measurement software
system runnable on lab computers. This includes Fricon, local runtime
components, data libraries, Python/package environments, configured code
sources, update paths, diagnostics, and migration support. Its expertise is
system readiness, not experiment design.

Goals:

- keep the local measurement software system runnable, including Fricon,
  runtime components, data libraries, Python/package environments, configured
  code sources, and update paths
- make setup, compatibility, migration, and support problems diagnosable
  before users read raw logs
- help labs adopt Fricon gradually while old LabRAD/Data Vault, folder, and
  parameter-file history remains in place

Context and pressure:

- supports machines that may be offline, locked down, slow to update, or
  pinned to lab-specific Python environments
- must handle sensitive local paths, machine names, IP addresses, environment
  details, and setup files carefully
- may diagnose OS, package, service, import, driver-path, and runner-launch
  problems from a software-system perspective
- contributes technical readiness checks when a proposal depends on runtime,
  update timing, data-library compatibility, or environment state

Common switches:

- works with P-005 to make authored methods runnable through maintained local
  code sources, package environments, and setup profiles
- relies on P-004 or P-003 for scientific acceptance of calibration,
  parameter, analysis, or interpretation outcomes

## P-003 Experimental Data Analyst

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
- switches to P-004 when analysis output becomes evidence for accepting,
  rejecting, or rolling back calibration-derived effective configuration or
  parameter state
- uses report/export recipes maintained by P-005

## P-004 Effective Configuration Steward

The role active when measurement evidence, analysis outputs, and calibration
results are evaluated for working effective configuration or parameter state.
This may happen before, during, or after calibration measurements, and the same
person may also be the measurement run operator or experimental data analyst.

Goals:

- decide which effective configuration or prior-good state calibration work
  depends on
- evaluate calibration evidence, fitted values, health gates, retry/pause
  needs, and manual inspection flags
- mark calibration results as exploratory, accepted, rejected, superseded, or
  needing review, with rollback context where practical
- preserve the distinction between parameter/configuration decisions and
  physical setup or device-state ownership

Context and pressure:

- works with mutable JSON files, lock files, dated backups, copied setting
  folders, generated temporary files, wiring sheets, and local database helpers
- may run routines that update local parameters while also producing datasets,
  so mutation and measurement facts must stay distinguishable
- may depend on physical instrument addresses and external device state that
  Fricon can record before it can control

Common switches:

- uses P-001 for the run-operation part of calibration work
- uses P-003 when fitted outputs, plots, or derived artifacts need analysis
  provenance before a calibration decision
- depends on P-005 for calibration routines and code provenance
- uses P-002 technical readiness checks when calibration proposals affect
  runtime, update, environment, or data-library compatibility

## P-005 Measurement Method Author

The role active when a lab user writes or maintains code that expresses a
measurement method, from exploratory script blocks to reusable routines:
scripts, scan helpers, pulse-generation rules, runner integrations, plotting
utilities, or export/report recipes. This role understands how the experiment
should be expressed in code, but may rely on P-002 for packaging,
environments, deployment, and local system diagnostics.

Goals:

- integrate Fricon recording into exploratory scripts, notebooks, Data
  Vault-style helpers, reusable routines, runners, and scan utilities
- declare scan axes, dependencies, trace shapes, repeated shots, validity
  masks, runner identifiers, and procedure summaries from code
- preserve honest code provenance without claiming Fricon managed execution
  guarantees it does not yet provide

Context and pressure:

- may write quick internal script blocks or routines that other users later run
  without understanding every dependency
- may rely on P-002 for runtime, package, deployment, or diagnostic support
- needs migration ergonomics that do not require rewriting the whole
  measurement stack at once
- creates post-MVP pressure for approved code sources, managed entry points,
  templates, and reviewed updates

Common switches:

- enables P-001 by making scripts and helpers easier to run and record
- helps P-003 produce traceable analysis and report artifacts
- helps P-004 produce calibration evidence and parameter-change proposals
- works with P-002 when methods need maintained code sources, package
  environments, setup profiles, or managed entry points
