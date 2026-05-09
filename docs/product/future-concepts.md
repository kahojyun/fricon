# Future Concepts

## Status

Draft strategic follow-on backlog; older future-story and concept IDs retired.

## Purpose

Preserve strategic follow-on pressure without letting older numbered epics,
stories, requirements, or capabilities inflate the initial adoption slice.

This file does not preserve old stable IDs. Fresh stable IDs can be introduced
later when a future concept is revalidated and needs durable traceability.
Items may be promoted only after the initial adoption baseline is accepted and
the product owner confirms the story, scope, validation need, and any ADR
boundary.

These concepts are future pressure, not dead backlog. During domain and
architecture analysis, use this file to check that first-slice decisions keep
room for Fricon's differentiating long-term loops: explain, compare, review,
repeat, and automate safely.

## Promotion Rule

A future concept can move toward implementation only when:

- it does not make the first write/watch/reopen loop harder
- it has a clear accountable user role
- required compatibility, storage, safety, or API decisions are captured in an
  ADR or later spec
- it is derived from current product evidence, not from old numbered backlog
  residue

## Use During Domain And Architecture Analysis

Future concepts should not drive first-slice implementation by themselves, but
they should constrain early abstractions. A first domain or architecture pass
should explicitly ask whether its measurement, dataset artifact, identity,
lifecycle, event, provenance, attachment, note, attribute, reader, and API
models can later support:

- records that cite source measurements and produced artifacts
- effective parameter snapshots and reviewed proposal history
- code provenance moving from unmanaged labels to managed sources and snapshots
- run manifests assembled from recorded facts
- calibration evidence and reviewable automation proposals
- setup/device observed facts and ADR-gated apply plans
- portable export and read-only bundle access

If an early design cannot support these without a conceptual rewrite, either
revise the design or record the tradeoff in an ADR before implementation.

## Strategic Follow-On Priorities

### Parameter System

Why it matters:

- Mutable parameter/config files are a major source of ambiguity in legacy
  work.
- Later managed runs and calibration proposals need effective parameter facts.
- Parameter diffs and snapshots are the realistic path toward repeatable
  experiment memory.

Candidate scope:

- named parameter profiles or refs
- immutable run-bound snapshots
- effective settings and overrides at run start
- diffs between runs, profiles, and proposals
- lightweight proposal flow with source evidence, actor, reason, approval
  history, before/after diff, and rollback target
- hybrid parameter tree: flexible values first, typed definitions only where
  they earn their complexity

Boundaries:

- no strict global registry as a first parameter slice
- no device write-back before safety and readback decisions
- no heavyweight permissions or compliance system by default
- no claim that physical setup notes are parameter truth

### Managed Code And Managed Run

Why it matters:

- Copied code folders are a weak provenance model.
- Managed run can improve context capture when users opt in.
- Calibration and repeat workflows need to cite code sources or snapshots.

Candidate scope:

- configured measurement-code sources
- local checkout or package source guidance
- importable Python entry points
- environment hints or lock files
- opt-in SDK runner integration
- code snapshot and runtime summary capture
- run status, logs, diagnostics, and produced artifacts
- run manifest assembled from existing facts

Boundaries:

- ordinary unmanaged Python remains valid and clearly labeled
- no blind shell-command runner as the main UX
- no scheduler, queue, resource lease, or workflow DAG as the first managed-run
  slice
- no resumable scan-point execution before a managed runner design exists

### Analysis And Handoff Records

Why it matters:

- Users already create fit results, plots, derived arrays, spreadsheets, and
  reports outside acquisition.
- Fricon can later make these derived artifacts traceable to source
  measurements, code, parameters, and manual judgment.

Candidate scope:

- analysis records that cite input measurements and produced artifacts
- derived-result records for fit outputs, extracted peaks, residuals, score
  surfaces, or classifier diagnostics
- interpretation notes and handoff state
- run comparison and failure investigation views

Boundaries:

- first adoption can leave analysis in notebooks and scripts
- no first-slice report or presentation generation
- no automatic notebook state capture as the normal provenance mechanism

### Calibration Evidence And Reviewable Automation

Why it matters:

- Calibration already mixes measurement evidence, fitted values, generated
  files, parameter edits, and operator judgment.
- Hidden mutation is the main product risk.

Candidate scope:

- calibration evidence records
- fitted values with affected parameter paths
- task health, retry/pause decisions, and manual inspection flags
- chain-scoped working refs for calibration progress
- reviewed promotion to durable parameter profiles
- previewable automation proposals with before/after diffs and audit events

Boundaries:

- no fully automatic durable parameter write-back before review paths exist
- no AI or automation mutation without explicit preview and audit
- no generic workflow DAG as the normal measurement model

### Setup And Device State

Why it matters:

- Setup, wiring, instrument state, and device readback can explain differences
  between runs.
- First adoption should not pretend to understand opaque setup files, but later
  structured setup/device state may be valuable.

Candidate scope:

- structured setup snapshots
- device identity and observed/readback facts
- desired-state planning for managed routines
- reconciliation diffs before apply
- apply plans with dependency, settle, readback, timeout, and abort semantics

Boundaries:

- device communication and mutation are ADR-gated
- setup/device state needs an accountable owner distinct from software-system
  readiness
- first adoption records notes, attributes, and selected files only

### Viewer, Export, And Remote Access

Why it matters:

- Users will want handoff, richer browsing, and comparison after local reopen is
  reliable.

Candidate scope:

- portable Fricon package export
- generic file exports when demand is validated
- read-only bundle viewer
- richer historical viewer and saved views
- trace concatenation helpers and coarse/fine overlay polish
- read-only LAN monitoring

Boundaries:

- local reopen comes first
- no distributed editable library semantics
- no full viewer polish as a first-slice blocker

### Sample Maps And Spatial Views

Why it matters:

- Some labs benefit from sample maps or target visualizers.

Candidate scope:

- richer sample metadata
- user-authored sample-map configs or DSLs
- saved visual query views over parameter snapshots or run results

Boundaries:

- sample visualization is a view, not proof Fricon understands physical sample
  geometry
- visualizer schema migration and invalid visualizer handling need later design

## Lower-Priority Or ADR-Gated Directions

- hosted collaboration
- distributed editable data libraries
- third-party protocol compatibility guarantees
- mutation-capable device apply
- resumable execution checkpoints
- AI-assisted mutation

## Deprecated Numbered Backlog

Earlier versions of this backlog used numbered future epics, stories,
requirements, and concept IDs. Those identifiers are deprecated and should not
be reused. Reintroduce any useful idea as current product text or a new
accepted spec item after revalidation. Add fresh IDs later only when the
revalidated boundaries need stable references.
