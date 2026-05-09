# Product Analysis Progress

## Status

Accepted as the product-analysis progress tracker.

## Purpose

Track analysis progress, confidence, open questions, and the next analysis
sequence before downstream domain, architecture, spec, or implementation
planning starts.

This file does not define product scope. Product promises belong in
`vision.md`; role ownership belongs in `personas.md`; current story and
capability structure belongs in `story-map.md` and `capability-map.md`.

## Current Situation

Fricon restarted product analysis after earlier planning became difficult to
continue from. Some current product documents still preserve useful lessons
from older planning, but the active baseline is being rederived from current
greenfield inputs.

Most reviewed inputs:

- `vision.md`
- `personas.md`
- three initial-adoption interview passes using a VNA S21 scan, IQ/readout
  data, generic irregular/minimizer records, and selected legacy-code pressure
- cleanup of older numbered epic, story, capability, and future-story artifacts
  into rederived maps and backlog categories

## Document Confidence

High-confidence product inputs:

- `product/vision.md`
- `product/personas.md`

Current draft product structure:

- `product/story-map.md`
- `product/capability-map.md`
- `product/python-sdk-ux.md`
- `product/glossary.md`
- `product/future-concepts.md`

Not ready for implementation use:

- `domain/`
- `architecture/`, except accepted reset constraints and compatibility policy
- `specs/`
- `implementation-plans/`

## Progress Snapshot

| Area | Current State | Next Action |
| --- | --- | --- |
| Problem framing | Strong; centered on replacing the Data Vault/Grapher loop for new interactive work | Keep as product thesis unless new evidence contradicts it. |
| Personas | Strong current role model | Refine only when new durable role pressure appears. |
| Initial adoption journey | Candidate journey complete from VNA/readout interview evidence | Validate against another concrete migration case. |
| Story map | Rewritten from current evidence; old IDs retired | Challenge first-slice boundaries. |
| Capability map | Rewritten from current story map; old IDs retired | Cross-check for gaps and excess against the first usable slice. |
| Scope definition | Narrower but still draft | Separate first usable slice, follow-on backlog, ADR-gated, and rejected scope. |
| Success signals | Missing | Define measurable product and validation signals. |
| Assumptions and risks | Partial | Add an explicit assumption and validation register. |
| Validation plan | Missing | Define interview, prototype, and migration-script checks. |
| Domain and architecture inputs | Deferred | Start only after product baseline and validation posture stabilize. |

## Current Next Action

The next active analysis step is scope separation:

1. Challenge the rederived first usable slice.
2. Separate first usable slice, follow-on backlog, ADR-gated directions, and
   rejected scope.
3. Add success signals, assumptions, risks, and validation tasks.

Do not reintroduce old numbered product IDs. Fresh IDs can be introduced later
when the rederived boundaries are accepted enough to need stable references.

## Future-Pressure Guardrail

The current interview evidence is intentionally first-adoption-heavy. That
focus should narrow the first usable slice, but it must not erase the long-term
product thesis. Strategic follow-on concepts are not implementation-ready
requirements, but domain and architecture analysis must treat them as pressure
that early models should not foreclose.

Keep these future pressures visible:

- parameter profiles, effective snapshots, diffs, proposals, and reviewed
  promotion
- managed code sources, code provenance levels, run capture, and run manifests
- analysis, fit, interpretation, handoff, and failure-investigation records
- calibration evidence, calibration chains, health gates, working refs, and
  reviewable automation proposals
- setup/device identity, observed state, desired-state planning, reconciliation
  diffs, and ADR-gated apply
- routine recipes, reviewed replay, and read-only compare or triage workflows
- portable export, bundle reading, richer historical viewer behavior, and
  remote or LAN monitoring
- sample-map views and AI-assisted reviewed actions

It is acceptable for future detail to remain in backlog form while early
design focuses on write/watch/reopen. It is not acceptable for the first
domain model, storage model, identity model, event model, or API boundary to
make these later systems impossible without a large conceptual rewrite.

## Interview Evidence Summary

### Round 1: Initial Adoption Case

Evidence came from a physical experimentalist migrating a legacy measurement
workflow, plus a redacted sample-code pressure check over VNA, readout,
optimizer, and local configuration patterns.

Candidate first adoption case:

- sweep DC voltage and VNA power from existing Python code
- record VNA-returned S21 traces
- let Fricon own measurement identity, dataset artifacts, simple live
  inspection, checkpoint-safe partial reads, and local reopen
- keep instrument calls, waveform generation, notebooks, plotting utilities,
  parameter files, and calibration helpers outside Fricon

Key data-shape pressure:

- regular 1D, 2D, and N-D scans are dominant
- traces have inner coordinates, and trace length or coordinate values may vary
  across records
- one record may carry multiple traces
- complex values should be first-class for magnitude/phase and I/Q views
- IQ data may appear as averages, explicit I/Q channels, single-shot arrays, or
  classified labels
- minimizer and optimizer output should start as generic irregular or ragged
  step records

Key context pressure:

- selected parameter, registry, wiring, demod/readout, sidecar, script, and
  notebook artifacts can be useful evidence
- Fricon should preserve selected files or summaries without claiming it can
  keep physical setup, wiring, or opaque external facts accurate
- setup and wiring context should remain notes, attributes, attachments, or
  opaque evidence unless a later model owns the semantics

Candidate success standard:

- Fricon is worth continuing when it runs reliably, records measurement
  identity and produced data, preserves written checkpoints after ordinary
  interruption, supports simple inspection, and lets users copy stable IDs or
  reader snippets for later Python analysis.

### Round 2: Trace Writing And Reopen UX

Refined expectations:

- prefer a dedicated trace-writing path, Labber-like in spirit, over forcing
  every trace into flat row appends
- support explicit coordinate/value arrays and compact regular-coordinate
  forms such as start/delta/value
- support multiple different traces within one outer sweep record
- expose complex data directly rather than reconstructing relationships from
  channel names
- make stable ID copy the fastest reopen path; richer reader, export, or plot
  snippets can live behind advanced menus
- treat whole-notebook capture as low-value for initial adoption because output
  cleanup, folder size, and variable-state recovery are poor
- avoid automatic stale/fresh/trusted judgments for opaque setup context

### Round 3: Reader Views And Analysis Tasks

Refined expectations:

- trace readers should start from analysis tasks: selected-trace line plots,
  coarse/fine comparison, and sweep-plus-trace heatmaps
- reader APIs should allow alternate views without committing internal storage
  to Polars, pandas, NumPy, or xarray
- Polars-like nested tables are plausible for record-centric trace tables
- pandas often works better as expanded/indexed interoperability output
- NumPy is most useful for dense arrays such as IQ shot tensors or regularized
  trace cubes
- xarray-like views are useful when axes are rectangular enough
- IQ single-shot reads should be ndarray-like where shape permits, with sweep
  dimensions first and shot dimension last
- minimizer output should remain generic step-table data first; best-parameter
  summaries can stay as later helper logic unless broader need validates them
- first adoption can keep attachments as a plain list
- Labber-like right-click menu actions plus keyboard shortcuts are a good model
  for fast stable-ID copy

Sample-code pressure:

- existing VNA analysis rebuilds heatmaps by sorting rows, deriving axes, and
  reshaping values; Fricon should make this easier when schema supports it and
  expose missing, duplicate, ragged, or incomplete cells
- coarse/fine trace use is a reader workflow that should preserve segment
  provenance
- IQ analysis needs raw shaped shot arrays and scatter/histogram inspection;
  durable classifier-tuning workflows can remain later derived-artifact
  pressure
- optimizer analysis needs visible step records more urgently than a special
  optimizer product model

## Open Questions

- Which reader view should be the first-contact default, and which should be
  alternate conversion methods?
- Should trace concatenation or coarse/fine overlay be a reader helper, a
  Desktop historical-inspection action, or both?
- Is any minimizer best-value summary general enough to promote later, or
  should this remain user helper code over generic step tables?
- Which validation case should challenge the VNA/readout first-slice
  boundaries?
- Which success signals prove that first adoption is good enough to support
  domain analysis?

## Cleanup Log

On 2026-05-09, older numbered epic, story, capability, story-module, and
future-story artifacts were removed from the active docs path. `story-map.md`
and `capability-map.md` now own the rederived story and capability baseline;
`future-concepts.md` owns strategic follow-on pressure without preserving old
future-story IDs.

## Downstream Guardrail

Do not use draft product documents to start domain modeling, architecture,
specs, or implementation planning. Downstream work should wait until the
greenfield analysis has accepted the relevant journeys, stories, capabilities,
scope boundaries, and validation posture.
