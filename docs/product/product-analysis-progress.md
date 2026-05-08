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
| Problem framing | Strong but should be summarized more sharply | Add a concise problem hypothesis if needed. |
| User and role analysis | Strong current baseline | Keep refining only when new case evidence appears. |
| Use case discovery | Partial | Reconstruct the initial adoption journey from `vision.md`, `personas.md`, and case evidence. |
| Alternatives and market analysis | Draft research synthesis exists | Use only for focused pressure, not product authority. |
| Value proposition | Clear internally | Write a short external-facing value statement later. |
| Core workflow | Partial | Define the initial adoption story backbone before editing capabilities. |
| Story map | Draft, mostly derived from older docs | Rebuild from the story backbone and role goals. |
| Capability map | Draft, mostly derived from older docs | Derive capabilities from accepted journeys and stories, then cross-check for gaps. |
| Scope definition | Draft despite detailed text | Separate initial adoption, strategic follow-on, ADR-gated, and rejected scope after story/capability analysis. |
| Initial adoption definition | Draft | Accept only after the journey, story map, and supporting capabilities cohere. |
| Success metrics | Missing | Define measurable product and validation signals. |
| Risks and assumptions | Partial | Add an explicit assumption and validation register. |
| Validation plan | Missing | Define interviews, prototype checks, and migration-script trials. |
| Product requirements | Not implementation-ready | Derive later from accepted journeys, stories, capabilities, and validation results. |
| Domain analysis | Deferred | Start only after the product analysis baseline is stable. |
| Architecture inputs | Deferred | Start only after product and domain baselines are stable. |

## Next Analysis Sequence

Use the current product documents as input evidence, not as the analysis order.
The next work should proceed from user work to product capabilities:

1. Write or confirm the concise problem hypothesis.
2. Define the initial adoption journey and story backbone.
3. Rebuild `product/story-map.md` from that backbone.
4. Recheck epics and user stories against the rebuilt story map.
5. Derive `product/capability-map.md` from the accepted stories.
6. Add success signals, assumptions, and validation tasks.

Capability review should follow story analysis. A capability without story or
journey support should be deferred, rewritten as a backlog hypothesis, or
rejected as old planning residue.

## Downstream Guardrail

Do not use draft product documents to start domain modeling, architecture,
specs, or implementation planning. Downstream work should wait until the
greenfield analysis has accepted the relevant journeys, stories, capabilities,
scope boundaries, and validation posture.
