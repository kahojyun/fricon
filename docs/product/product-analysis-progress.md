# Product Analysis Progress

## Status

Accepted as the product-analysis progress tracker.

## Purpose

Track how far the greenfield product analysis has actually progressed, and
which product documents are trusted inputs versus provisional material.

This file does not define product scope. It records confidence, open work, and
the order in which product documents should be revalidated.

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
are revalidated from those two inputs and the current greenfield questions.

## Confidence Levels

High-confidence product inputs:

- `product/vision.md`
- `product/personas.md`

Provisional product helpers:

- `product/glossary.md`
- `product/python-sdk-ux.md`

Draft derived product artifacts pending revalidation:

- `product/capability-map.md`
- `product/story-map.md`
- `product/epics/`
- `product/user-stories/`

Backlog material only:

- `product/future-concepts.md`
- `product/future-stories-and-requirements.md`

Downstream material not ready for implementation use:

- `domain/`
- `architecture/`, except the accepted reset constraints and compatibility
  policy
- `specs/`
- `implementation-plans/`

## Greenfield Analysis Progress

| Step | Current State | Next Action |
| --- | --- | --- |
| Problem framing | Strong but should be summarized more sharply | Add a concise problem hypothesis if needed. |
| User and role analysis | Strong current baseline | Keep refining only when new case evidence appears. |
| Use case discovery | Draft, mostly derived from older docs | Rebuild through capability and story-map review. |
| Alternatives and market analysis | Draft research synthesis exists | Use only for focused pressure, not product authority. |
| Value proposition | Clear internally | Write a short external-facing value statement later. |
| Scope definition | Draft despite detailed text | Revalidate MVP, post-MVP, and rejected scope from current inputs. |
| Core workflow | Draft | Rebuild after capability revalidation. |
| Capability map | Next critical target | Review each capability as keep, revise, defer, or reject. |
| MVP definition | Draft | Accept only after capability and story map are rederived. |
| Success metrics | Missing | Define measurable product and validation signals. |
| Risks and assumptions | Partial | Add an explicit assumption and validation register. |
| Validation plan | Missing | Define interviews, prototype checks, and migration-script trials. |
| Product requirements | Not implementation-ready | Derive later from revalidated capabilities and stories. |
| Domain analysis | Deferred | Start only after product revalidation. |
| Architecture inputs | Deferred | Start only after product and domain baselines are stable. |

## Current Revalidation Rule

Do not use draft product documents to start domain modeling, architecture, specs,
or implementation planning.

The next product-analysis task should be a systematic review of
`product/capability-map.md`. For each capability, decide:

- Keep: directly supported by `vision.md`, `personas.md`, and current case
  evidence.
- Revise: direction is useful but wording, scope, or boundary is stale.
- Defer: plausible future direction, but not MVP product baseline.
- Reject: old planning residue or inconsistent with the current direction.

After that review, rebuild `product/story-map.md`, then revalidate epics and
user stories from the revalidated capability set.
