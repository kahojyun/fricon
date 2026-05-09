# Product Vision

## Status

High-confidence product input; derived scope pending revalidation.

## Purpose

State Fricon's product thesis, planning horizons, user promise, and durable
boundaries. Detailed story slices, capabilities, SDK guidance, interview
evidence, and strategic backlog details live in their owning documents.

## Thesis

Fricon is a local measurement data system for scientific experiment work.

The first product target is a maintained replacement for the fragile
Data Vault/Grapher loop around new interactive measurements. The long-term
product should become the lab's local experiment memory and reviewed action
layer: a system of record for measurement intent, effective parameters, code
provenance, setup state, analysis attempts, trust decisions, handoff, and
reviewed replay.

## Problem

Experimental measurement work is hard to preserve when data, parameters,
measurement code, setup state, notes, and later analysis live in separate tools
or informal files. Existing frameworks can collect data, but the broader
experiment record often depends on folder conventions, copied code, mutable
JSON files, sidecars, notebooks, and operator memory.

Notebook-based exploration is not the problem. The failure appears when useful
work has to survive beyond one person, one lab computer, one copied code
folder, one package environment, or one uninterrupted storage session.

## Planning Horizons

Initial Adoption Slice is the first practical release path: record new
interactive measurement data from ordinary Python, monitor simple live views,
keep already-written data readable after ordinary interruptions, and reopen
measurements or datasets by stable ID. Export remains important, but follows a
reliable local reopen path.

Strategic Follow-On covers product-core capabilities that remain central to
Fricon's thesis but depend on more product evidence, domain modeling,
architecture decisions, or safety and review boundaries. Parameter management,
managed run, calibration evidence, run manifests, setup/device state, and
reviewed automation belong here by dependency, not by importance.

ADR-Gated directions may be valuable but need explicit compatibility, safety,
data-semantics, or architecture decisions before durable product scope. Device
mutation, setup/device apply, resumable execution, distributed editable
libraries, and AI-assisted mutation are examples.

## Product Promise

For initial adoption, Fricon should help a researcher answer:

- What measurement did I run?
- What datasets and attachments did it produce?
- Was it finished, interrupted, failed, invalidated, or corrected?
- Which notes, user-supplied attributes, code context, and selected local files
  explain it?
- How do I inspect it live, reopen it from Python, or read already-written data
  after an ordinary interruption?

Strategic follow-on work should extend those facts so users can also ask:

- What changed since the previous good run?
- Which parameter, setup, code, analysis, or calibration facts explain the
  difference?
- Which code source and parameter snapshot did calibration depend on?
- What will change before a routine, device apply, or automation mutates
  durable state?
- What evidence, review decision, and handoff state did Fricon record
  afterward?

## Adoption Stance

Fricon is intended for gradual lab adoption. Labs should be able to keep old
runners, notebooks, LabRAD/Data Vault history, local parameter files, generated
sidecars, and folder-based workflows while new measurement work moves into
Fricon.

Legacy mechanisms should enter Fricon as aliases, context, attachments,
summaries, or evidence rather than as first-class product models. Selected
mutable configuration files can start as run-bound evidence and later become
effective snapshots or reviewed proposals. Copied folders can start as honest
code provenance and later become managed code sources. Calibration scripts can
start as ordinary measurements and later become calibration evidence.

If a legacy need cannot fit one of these bridge forms, it should not become an
initial adoption concept without explicit product and architecture review.

## Mental Models

Initial adoption:

```text
I ran a measurement from Python.
It produced datasets.
Fricon helps me monitor, inspect, and reopen them.
```

Strategic follow-on:

```text
I can choose a previous-good run or routine.
Fricon shows the effective parameters, code, setup, and expected artifacts.
I review what will change before anything durable is mutated.
Afterward, Fricon records the outcome, evidence, and handoff state.
```

## Initial Adoption Boundaries

The first usable slice should include:

- one local data library per normal lab computer, with coherent Desktop, CLI,
  Python SDK, local-runtime, update, and compatibility behavior
- unmanaged Python measurement recording with low ceremony
- first-class measurements and directly openable dataset artifacts
- common 1D, 2D, and N-D sweeps, trace-valued records, first-class complex
  values, IQ-like data, and generic irregular or ragged step records
- nonblocking live monitor views for simple line/scatter, heatmap, selected
  output or trace channels, and IQ scatter
- checkpoint-safe readability for already-written data after user interruption
  or notebook-kernel failure, without claiming hard-crash or power-loss
  recovery beyond later durable-write decisions
- lightweight notes, attributes, attachments, events, favorites/pins, and
  correction history
- honest software-visible context such as unmanaged code labels, optional
  script or source-folder references, selected local configuration files,
  environment labels, and procedure summaries
- stable-ID reopen through public Python APIs and generic reader output that
  users can wrap in experiment-specific helpers
- migration guidance for simple Data Vault-style recording-section rewrites

The first slice should not require old-history import, LabRAD emulation,
managed execution, device control, report generation, user plotting-code
execution, parameter profiles, calibration automation, or polished portable
export.

## Product Pressures

Keep these pressures visible until architecture or spec work owns them:

- Lab computers are often Windows, offline, locked down, or updated slowly.
  Fricon should behave like one coherent local product, not a manually
  assembled set of services and packages.
- Dataset semantics should be checked against real measurement shapes,
  including adaptive points, instrument-tuned traces, per-record trace axes,
  repeated shots, and incomplete grids.
- Required metadata should stay limited to facts needed for later
  interpretation, slicing, and plotting. Optional labels, units, notes,
  sample/session context, operator labels, and free-form JSON should not block
  ordinary writes.
- Physical setup, wiring, arbitrary registry files, and lab-environment facts
  that Fricon does not understand should remain user-supplied notes,
  attributes, attachments, or opaque evidence.
- Mutating actions should be attributable, previewable, and reviewable before
  Fricon changes durable parameter, setup, device, or data-library state.

## Strategic Follow-On Direction

Strategic follow-on should extend initial adoption facts into three loops:

- Explain: run manifests, lifecycle evidence, parameter/code/setup provenance,
  analysis attempts, and failure or anomaly investigation.
- Compare: previous-good baselines, drift signals, operator handoff,
  analysis/calibration status, and visible differences before repeat work.
- Repeat: run-like-previous drafts, reviewed parameter proposals, routine
  recipes, and audited automation after recorded facts are trustworthy.

The current priority hypothesis is parameter system first, managed run second,
and calibration chains with reviewable automation third. This ordering is not
accepted implementation scope until later product analysis validates it.

Detailed strategic backlog pressure lives in `future-concepts.md`; the
interview and validation state lives in `product-analysis-progress.md`.

## Non-Goals For Initial Adoption

- hosted SaaS, accounts, teams, permissions, or distributed editable libraries
- LabRAD compatibility server, LabRAD-dependent helper module, or built-in
  legacy Data Vault import/browser
- broad device-driver framework, visual sweep builder as the primary
  acquisition model, or generic workflow DAG engine
- automatic notebook state capture or managed code deployment
- hard-crash or power-loss recovery beyond accepted durable-write behavior
- parsing arbitrary setup, wiring, registry, or parameter files into trusted
  Fricon-owned truth
- polished portable export before local reopen works well
- AI or automation that mutates data-library state without explicit review and
  audit
