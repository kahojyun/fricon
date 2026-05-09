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
- a first interview pass on the initial adoption journey, using a simple VNA
  S21 scan and related readout/minimizer cases as redacted case evidence.

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
- `product/epics/`
- `product/user-stories/`

Strategic follow-on backlog material:

- `product/future-concepts.md`
- `product/future-stories-and-requirements.md`

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
| Use case discovery | In progress | Continue interview from the VNA/readout first-adoption case and resolve the open journey questions below. |
| Alternatives and market analysis | Draft research synthesis exists | Use only for focused pressure, not product authority. |
| Value proposition | Clear internally | Write a short external-facing value statement later. |
| Core workflow | Candidate backbone drafted | Validate whether the VNA/readout backbone is general enough for the first adoption story. |
| Story map | Draft, first interview pass added | Rebuild from the accepted story backbone and role goals after the next interview pass. |
| Capability map | Draft, mostly derived from older docs | Derive capabilities from accepted journeys and stories, then cross-check for gaps. |
| Scope definition | Draft despite detailed text | Separate initial adoption, strategic follow-on, ADR-gated, and rejected scope after story/capability analysis. |
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
The next work should proceed from user work to product capabilities:

1. Define the initial adoption journey and story backbone.
2. Rebuild `product/story-map.md` from that backbone.
3. Recheck epics and user stories against the rebuilt story map.
4. Derive `product/capability-map.md` from the accepted stories.
5. Add success signals, assumptions, and validation tasks.

Capability review should follow story analysis. A capability without story or
journey support should be deferred, rewritten as a backlog hypothesis, or
rejected as old planning residue.

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
  partial-read, reopen, and export loop while instrument calls, waveform
  generation, LabRAD-era services, notebooks, plotting utilities, parameter
  files, and calibration helpers can remain outside Fricon

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

Open questions for the next interview pass:

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

## Downstream Guardrail

Do not use draft product documents to start domain modeling, architecture,
specs, or implementation planning. Downstream work should wait until the
greenfield analysis has accepted the relevant journeys, stories, capabilities,
scope boundaries, and validation posture.
