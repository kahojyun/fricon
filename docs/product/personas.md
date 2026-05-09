# Personas

## Status

Accepted.

## Purpose

Define durable product-role archetypes for Fricon planning. Personas route
ownership of goals, risks, and decisions; they are not fictional biographies,
permissions, feature requirements, or replacements for stories and specs.

Use `story-map.md`, `capability-map.md`, specs, ADRs, or domain docs for
concrete behavior and acceptance criteria.

## Grounding

These roles were refined against representative lab practice: Windows lab
folders, Jupyter notebooks, LabRAD Data Vault, mutable parameter and registry
files, wiring spreadsheets, generated sidecars, calibration scripts, hardware
bring-up helpers, backups, and report artifacts.

They are responsibility views. One person may move between several roles in
one day.

## Routing Rules

Choose the persona that owns the main outcome:

- P-001 Measurement Run Operator owns ordinary measurement operation and
  checkpoint-safe access to already-written data after ordinary interruptions.
- P-002 Local Measurement System Maintainer owns Fricon deployment, local
  runtime health, data-library readiness, Python/package environment support,
  diagnostics, migration support, and technical readiness.
- P-003 Experimental Data Analyst owns reopening, analysis, derived artifacts,
  downstream handoff, and interpretation provenance.
- P-004 Effective Configuration Steward owns effective configuration,
  calibration evidence, parameter state, fitted values, and rollback decisions.
- P-005 Measurement Method Author owns measurement-method code, SDK usage, scan
  helpers, runner integration, plotting utilities, and export/report recipes.

If a workflow mutates durable lab state, the accountable role is not only
P-001. Route parameter or effective-configuration decisions to P-004, managed
routine or measurement-code shape to P-005, durable analysis or interpretation
records to P-003, and runtime/update/library safety to P-002.

Physical setup, instrument, wiring, and device-state ownership is a distinct
future boundary. Initial adoption records those facts as notes, attributes,
attachments, or run-bound local configuration evidence. Later workflows that
review or mutate setup/device state need an explicit setup/device owner rather
than defaulting to P-002.

## First Adoption Emphasis

The first adoption slice is primarily for P-001 and P-003: an experimenter
runs an interactive script or notebook, then reopens the result by stable ID
for later analysis. P-005 and P-002 are supporting roles for migration
ergonomics and local-system readiness.

P-004 remains strategically important, but first adoption treats calibration
notebooks as ordinary measurement work unless later evidence shows a minimal
calibration-specific record removes real user burden.

## P-001 Measurement Run Operator

The role active when a lab user runs and monitors a measurement script or
notebook on a lab computer.

Goals:

- start, watch, annotate, preserve, and later hand off measurement runs
- choose run-specific inputs such as scan ranges, selected targets, and context
  labels
- understand measurement identity, scan shape, units, and selected context
  without learning Fricon internals
- keep already-written data readable after user interruption or notebook-kernel
  failure

Context:

- often works on Windows, offline, locked-down, or slow-to-update machines
- depends on Jupyter, package environments, LabRAD/Data Vault services, copied
  folders, dated backups, and mutable local configuration during migration
- needs ordinary measurement work to become safer without adopting managed
  execution first
- may run multiple independent measurements at once

Common switches:

- P-003 for analysis and interpretation after acquisition
- P-004 for judging calibration evidence or fitted values
- P-005 for changing measurement-method code or dataset schema
- P-002 when setup, update, or runtime problems block work

## P-002 Local Measurement System Maintainer

The role active when someone makes the local Fricon-backed measurement
software system runnable on lab computers. Its expertise is system readiness,
not experiment design.

Goals:

- keep Fricon, local runtime components, data libraries, Python environments,
  configured code sources, and update paths runnable
- make setup, compatibility, migration, and support problems diagnosable before
  users read raw logs
- support gradual adoption while old LabRAD/Data Vault, folder, and
  parameter-file history remains in place

Context:

- handles machines that may be offline, locked down, pinned, or slow to update
- must treat local paths, machine names, IP addresses, environment details, and
  setup files as sensitive
- diagnoses OS, package, service, import, driver-path, runner-launch, library,
  and update-timing problems from a software-system perspective

Common switches:

- works with P-005 to make authored methods runnable through maintained code
  sources, packages, environments, and setup profiles
- relies on P-004 or P-003 for scientific acceptance of calibration, parameter,
  analysis, or interpretation outcomes

## P-003 Experimental Data Analyst

The role active when someone reopens completed or interrupted measurements for
notebooks, local analysis, HPC analysis, reports, or handoff.

Goals:

- reopen measurements through stable IDs and public APIs
- load data into analysis-friendly Python objects where appropriate
- preserve links from derived artifacts back to source measurements,
  parameters, code/procedure context, and manual judgment
- distinguish recorded measurement facts from notebook-local analysis state
  and manually edited outputs
- later, use portable exports without recreating the acquisition-time runtime

Context:

- often works away from the acquisition computer or after the run finished
- needs privacy-aware handoff when exports contain local paths, setup details,
  code summaries, environment data, or sample context
- can keep screenshots, exported images, and custom plotting scripts outside
  Fricon until real demand justifies more

Common switches:

- consumes measurements produced by P-001
- switches to P-004 when analysis output becomes calibration or parameter
  evidence
- uses report/export recipes maintained by P-005

## P-004 Effective Configuration Steward

The role active when measurement evidence, analysis outputs, and calibration
results are evaluated for effective configuration or parameter state.

Goals:

- decide which effective configuration or prior-good state future work depends
  on
- evaluate calibration evidence, fitted values, health gates, retry/pause
  needs, and manual inspection flags
- mark calibration results as exploratory, accepted, rejected, superseded, or
  needing review
- keep parameter/configuration decisions distinct from physical setup or
  device-state ownership

Context:

- works with mutable JSON files, lock files, dated backups, copied setting
  folders, generated temporary files, wiring sheets, and local database helpers
- may run routines that update local parameters while producing datasets, so
  mutation and measurement facts must stay distinguishable
- may depend on physical instrument addresses and external device state that
  Fricon can record before it can control

Common switches:

- uses P-001 for run operation
- uses P-003 for fit, plot, and derived-artifact provenance
- depends on P-005 for calibration routines and code provenance
- uses P-002 when proposals affect runtime, update, environment, or
  data-library compatibility

## P-005 Measurement Method Author

The role active when someone writes or maintains the code that expresses a
measurement method: scripts, scan helpers, pulse-generation rules, runner
integrations, plotting utilities, reusable routines, or export/report recipes.

Goals:

- integrate Fricon recording into existing exploratory scripts, notebooks,
  Data Vault-style helpers, routines, runners, and scan utilities
- declare scan axes, dependencies, trace shapes, repeated shots, validity masks,
  runner identifiers, and procedure summaries from code
- preserve honest code provenance without claiming managed execution guarantees
  Fricon does not yet provide
- help users migrate by rewriting recording sections with small, explicit
  Fricon calls rather than a built-in LabRAD compatibility layer

Context:

- may write quick internal script blocks or routines that other users later run
  without understanding every dependency
- may rely on P-002 for runtime, packaging, deployment, and diagnostics
- creates strategic follow-on pressure for approved code sources, managed entry
  points, templates, reviewed updates, and managed runs

Common switches:

- enables P-001 by making scripts easier to run and record
- helps P-003 produce traceable analysis and report artifacts
- helps P-004 produce calibration evidence and parameter-change proposals
- works with P-002 on maintained code sources, package environments, setup
  profiles, and managed entry points
