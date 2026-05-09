# Product Analysis Progress

## Status

Accepted as the product-analysis progress tracker.

## Purpose

Track how far the greenfield product analysis has actually progressed, and
which analysis questions still need work before downstream domain,
architecture, spec, or implementation planning starts.

This file does not define product scope. It records analysis progress,
confidence, open questions, and the next analysis sequence.

## Current Situation

Fricon restarted product analysis after earlier planning became difficult to
continue from. Some current product documents were extracted from older
planning material. They may contain useful lessons, but they were not all
derived systematically from the current greenfield direction.

The most carefully reviewed current inputs are:

- `product/vision.md`, refined from the current clean-reset direction and the
  user-supplied sample codebase review.
- `product/personas.md`, refined from the same case-study pressure and current
  product-role thinking.
- three interview passes on the initial adoption journey, using a simple VNA
  S21 scan and related readout/minimizer cases as redacted case evidence.
- a cleanup pass that replaced older numbered epic, story, capability, and
  future-story artifacts with unnumbered active maps and backlog categories.

Other product documents should be treated as draft derived material until they
are rederived or checked against the current greenfield analysis.

## Document Confidence

High-confidence product inputs:

- `product/vision.md`
- `product/personas.md`

Provisional product helpers:

- `product/glossary.md`
- `product/python-sdk-ux.md`

Draft derived product artifacts:

- `product/capability-map.md`
- `product/story-map.md`
- `product/old-numbered-artifacts.md` for deprecation context only

Strategic follow-on backlog material:

- `product/future-concepts.md`
- `product/future-stories-and-requirements.md` is deprecated and points to the
  unnumbered future backlog

Downstream material not ready for implementation use:

- `domain/`
- `architecture/`, except the accepted reset constraints and compatibility
  policy
- `specs/`
- `implementation-plans/`

## Greenfield Analysis Progress

| Step | Current State | Next Analysis Action |
| --- | --- | --- |
| Problem framing | Strong; sharpened around maintained Data Vault/Grapher replacement for new interactive work | Rebuild the initial adoption story backbone from this framing. |
| User and role analysis | Strong current baseline with first-adoption emphasis on P-001 and P-003 | Keep refining only when new case evidence appears. |
| Use case discovery | Candidate journey complete and old IDs removed | Use the ID-free story map and capability map as the next product-analysis baseline. |
| Alternatives and market analysis | Draft research synthesis exists | Use only for focused pressure, not product authority. |
| Value proposition | Clear internally | Write a short external-facing value statement later. |
| Core workflow | Candidate backbone strengthened and story-checked | Validate whether the VNA/readout backbone is general enough for the first adoption story. |
| Story map | Rewritten without old story or epic IDs | Challenge first-slice scope and validate against another concrete migration case before accepting. |
| Capability map | Rewritten without old capability IDs | Cross-check for gaps and excess against the ID-free story map. |
| Scope definition | Narrower but still draft | Separate first usable slice, follow-on backlog, ADR-gated, and rejected scope after capability review. |
| Initial adoption definition | Draft with first success standard | Accept only after the journey, story map, and supporting capabilities cohere. |
| Success metrics | Missing | Define measurable product and validation signals. |
| Risks and assumptions | Partial | Add an explicit assumption and validation register. |
| Validation plan | Missing | Define interviews, prototype checks, and migration-script trials. |
| Product requirements | Not implementation-ready | Derive later from accepted journeys, stories, capabilities, and validation results. |
| Older backlog review | Partial | Keep strategic follow-on docs as backlog only; revalidate managed run, calibration, Git-heavy provenance, report artifacts, and generic export formats before promoting them. |
| Domain analysis | Deferred | Start only after the product analysis baseline is stable. |
| Architecture inputs | Deferred | Start only after product and domain baselines are stable. |

## Next Analysis Sequence

Use the current product documents as input evidence, not as the analysis order.
The work should proceed from user work to product capabilities:

1. Define the initial adoption journey and story backbone.
2. Rebuild `product/story-map.md` from that backbone.
3. Rebuild `product/capability-map.md` from the story map.
4. Separate first usable slice, follow-on backlog, ADR-gated, and rejected scope.
5. Add success signals, assumptions, and validation tasks.

Steps 1 through 3 now have a candidate ID-free pass from the VNA/readout
interview backbone. Step 4 is the next active analysis step.

Capability review should follow story analysis. A capability without story or
journey support should be deferred, rewritten as an unnumbered backlog
hypothesis, or rejected as old planning residue. Do not reintroduce old
numbered product IDs.

## Future-Pressure Guardrail

The current interview evidence is intentionally first-adoption-heavy. That
focus should narrow the first usable slice, but it must not erase the long-term
product thesis. Strategic follow-on concepts are not implementation-ready
requirements, but domain and architecture analysis must treat them as pressure
that early models should not foreclose.

Keep these future pressures visible during domain and architecture work:

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

## Interview Evidence Log

### 2026-05-09: Initial Adoption Journey, Round 1

Evidence type:

- user interview as a physical experimentalist migrating a legacy measurement
  workflow gradually
- redacted sample-code pressure check over legacy VNA, readout, optimizer, and
  local configuration patterns

Candidate first adoption case:

- a simple VNA S21 measurement where existing Python code sweeps DC voltage and
  VNA power while recording VNA-returned S21 traces
- Fricon replaces the measurement record, dataset artifact, live inspection,
  checkpoint-safe partial-read, and reopen loop while instrument calls,
  waveform generation, LabRAD-era services, notebooks, plotting utilities,
  parameter files, and calibration helpers can remain outside Fricon. Export
  follows local reopen rather than defining the first adoption path.

Observed data-shape pressure:

- regular 1D, 2D, and N-D scans remain the dominant path
- VNA traces need an inner coordinate axis, commonly frequency, while outer
  sweep axes such as bias or power describe each trace record
- trace length and trace coordinate values may vary across records
- IQ readout may appear as averaged complex-like values, explicit I/Q channels,
  single-shot arrays, or classified labels
- IQ scatter inspection is a first-slice live and historical view need
- minimizer or optimizer work should have an easy irregular step-record shape
  so candidate parameters, objective values, status, and best-so-far state do
  not remain print-only evidence

Observed context pressure:

- selected parameter, registry, wiring, demod/readout, sidecar, script, and
  notebook artifacts are useful evidence around a run
- first adoption should bind selected local configuration as original files,
  summaries, or opaque evidence, not claim that Fricon can keep physical setup,
  wiring, or external environment facts accurate and current
- setup and wiring context is especially likely to be incomplete, stale, or
  reported later in slides or notes; this should surface as ambiguity rather
  than become a false reproducibility promise

Candidate success standard:

- Fricon is worth continuing to use for new runs when it runs reliably, records
  measurement identity and produced data, preserves written partial results,
  makes the data easy to inspect in the browser/viewer, and lets the user copy
  a measurement or dataset reader snippet for later Python analysis.

Questions carried into Round 2:

- Which exact first-contact writer shape is acceptable for a VNA trace inside
  an outer voltage/power sweep?
- Should the product model expose complex values directly, or mainly expose
  I/Q plus optional magnitude/phase views?
- What is the minimum useful table returned by the Python reader for trace
  datasets, IQ arrays, and irregular minimizer records?
- Which attached files should be shown as trusted run evidence, and which
  should be shown only as possibly relevant context?
- What should the browser show when setup context is missing, stale, or
  ambiguous, without turning everything into unstructured notes?
- Which UI copy actions are required for first adoption: copy measurement
  reader snippet, copy dataset reader snippet, copy stable ID, or copy export
  reader snippet?

### 2026-05-09: Initial Adoption Journey, Round 2

Evidence type:

- follow-up user interview focused on trace writing, complex values, reader
  ergonomics, context evidence, browser display, and copy actions

Refined product expectations:

- A dedicated trace-writing path is preferred over forcing all trace data into
  flat row appends. The user expects a Labber-like trace concept where a record
  can carry trace data as explicit coordinate/value arrays or as compact
  start/delta/value data.
- The writer shape should handle multiple different traces within the same
  outer sweep record.
- Complex values should be first-class product data so Fricon can provide
  targeted magnitude/phase views without asking users or plots to infer
  relationships from channel naming conventions.
- The fastest reopen path may only need a stable ID, or a small open-reader
  snippet with the ID as the input that users edit later. After analysis code
  stabilizes, users mostly change the input ID rather than the reopen code.
- The most realistic run-bound context attachments are mutable parameter files
  and instrument information. Whole-notebook capture is low-value for initial
  adoption because outputs can bloat measurement folders, cleaning outputs is
  hard, and the saved notebook still may not recover the variable state users
  actually need.
- First adoption should be skeptical about Fricon detecting stale or ambiguous
  setup/configuration facts. If Fricon does not understand user-provided setup
  context, it should avoid pretending it can judge freshness or correctness.
- Fast-path UI copy should prioritize copying stable IDs, preferably with
  keyboard shortcuts. Richer reader, export, or plot snippets can live behind
  advanced menus.

Round 2 implications that were incorporated into the ID-free story map:

- trace-valued records and multiple traces per record are product pressure
  without accepting exact API syntax
- complex-aware magnitude/phase views and IQ scatter are native inspection
  expectations
- stable ID copy is a first-path action, while reader snippets can be advanced
  actions
- automatic stale/ambiguous setup judgments are out of scope unless Fricon has
  explicit evidence to support them

Remaining open questions:

- Which concrete trace-reader views are needed first for VNA traces: nested
  trace objects, table with array columns, xarray-like data, or all of these as
  alternate views?
- What is the minimum acceptable reader shape for IQ single-shot arrays and
  minimizer step records?
- Should first adoption include a lightweight context role label such as
  trusted run evidence, user-provided context, or opaque attachment, or is a
  plain attachment list enough?
- Which keyboard shortcuts and browser locations should expose fast stable-ID
  copy?

### 2026-05-09: Initial Adoption Journey, Round 3

Evidence type:

- follow-up user interview focused on reader views, IQ/minimizer read shapes,
  attachment roles, and browser copy UX
- focused framework pressure check over common Python analysis libraries
- redacted sample-code pressure check over legacy trace plotting,
  heatmap-building, IQ analysis, classifier tuning, and optimizer analysis

Refined product expectations:

- VNA trace reading should start from user analysis tasks, not from cloning the
  old pipeline. Important tasks include plotting selected sweep traces in one
  line plot, combining or comparing coarse/fine trace ranges, and building 2D
  heatmaps from sweep-plus-trace data.
- The reader should support a natural query path for trace-bearing tables.
  Nested table views are promising for Polars-like workflows, while pandas and
  NumPy users may need expanded table or array-oriented views.
- Reader APIs should expose alternate views rather than force one shape for
  every analysis: record-centric nested trace tables, exploded trace tables,
  sample-level long tables, ndarray-oriented views, and xarray-like views when
  the data is rectangular enough.
- IQ single-shot data should be easy to read as ndarray-like data with sweep
  dimensions first and the shot dimension last where shape permits. This
  supports SNR analysis, error-rate estimation, and classifier-parameter
  improvement with NumPy-style processing.
- Minimizer records can stay simpler than measurement traces. The first useful
  read shape is a generic step table for plotting parameter and objective
  evolution. Best-parameter summaries by outer sweep condition can stay as
  later helper logic unless a broader reader need validates them.
- A plain attachment list is enough for first adoption. Lightweight attachment
  role labels can wait until evidence shows users need them.
- Labber-like right-click menu actions plus keyboard shortcuts are a good model
  for fast stable-ID copy in the browser.

Framework pressure check:

- Polars-style nested list, array, and struct data is a plausible fit for
  record-centric trace tables with queryable nested payloads.
- pandas is better treated as an interoperability view, often expanded or
  indexed, rather than as the only natural home for ragged nested traces.
- NumPy is most useful for dense arrays such as IQ shot tensors or regularized
  trace cubes, not for all reader-facing tabular metadata.
- xarray-like labeled data is useful when traces form rectangular
  multidimensional arrays with shared coordinates, but it should not be the
  only view for ragged or per-record trace axes.

Sample-code pressure check:

- Existing VNA analysis rebuilds heatmaps by sorting tabular rows, deriving
  unique axes, and reshaping measured values; Fricon should make this easier
  when schema supports it and should expose missing, duplicate, ragged, or
  incomplete cells explicitly.
- Coarse/fine trace use is a reader workflow: combine, compare, crop, fit, or
  overlay trace ranges while preserving segment provenance.
- IQ analysis often rotates and projects complex shot clouds, builds
  histograms, fits lobes, estimates SNR/error/fidelity, and adjusts classifier
  parameters. First adoption should make the raw shaped shot arrays and
  scatter/histogram inspection easy; durable classifier-tuning workflows can
  remain later analysis or derived-artifact pressure.
- Optimizer analysis needs visible step records and selected best values more
  urgently than a heavy optimizer-specific product model.

Round 3 implications that were incorporated into the ID-free story map:

- reader expectations should describe trace, IQ, and generic irregular data
  without committing the internal storage model to Polars, pandas, NumPy, or
  xarray
- right-click menu and keyboard shortcut copy actions are good browser UX for
  stable measurement and dataset IDs
- first adoption should keep attachments as a plain list rather than
  introducing attachment role taxonomy

Remaining open questions:

- Which reader views must be first-contact defaults, and which can be
  alternate conversion methods?
- Should trace concatenation/coarse-fine overlay be a reader helper, a Desktop
  historical inspection action, or both?
- Is any minimizer best-value summary general enough to promote later, or
  should this remain user helper code over generic step tables?

## Story And Capability Cleanup Log

### 2026-05-09: Remove Older Draft IDs

Scope:

- replaced older numbered epic, story, capability, and story-module artifacts
  with unnumbered active maps
- collapsed numbered strategic follow-on stories and requirements into an
  unnumbered future backlog
- kept a short deprecation note in `old-numbered-artifacts.md`

Cleanup outcomes:

- `story-map.md` now owns the current unnumbered story backbone.
- `capability-map.md` now owns the current unnumbered capability baseline.
- Older numbered files under product epics, user stories, and the story-module
  matrix are removed from the active docs path.
- `future-concepts.md` now owns unnumbered strategic follow-on categories.
- `future-stories-and-requirements.md` is deprecated.

Next analysis action:

- challenge and validate the ID-free first usable slice, then add success
  signals, assumptions, and validation tasks.

## Downstream Guardrail

Do not use draft product documents to start domain modeling, architecture,
specs, or implementation planning. Downstream work should wait until the
greenfield analysis has accepted the relevant journeys, stories, capabilities,
scope boundaries, and validation posture.
