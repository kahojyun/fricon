# Agent Steering

## Status

Draft.

## Routing Rules

| Task | Start With |
| --- | --- |
| Product direction | `product/product-analysis-progress.md`, then `product/vision.md` and `product/personas.md` |
| Product term or concept question | `product/product-analysis-progress.md`, then draft `product/glossary.md` |
| Architecture question | `architecture/README.md`, then revalidated product docs |
| Storage/API/protocol decision | First confirm product inputs; use `architecture/README.md` and ADRs, and do not create detailed architecture files until architecture design starts |
| Compatibility question | `architecture/compatibility-policy.md`, `decisions/ADR-001-v02-clean-reset-boundary.md` |
| Implementation planning | First confirm revalidated product inputs plus accepted architecture/ADR inputs, then recreate or read derived `specs/` and `implementation-plans/`; they are sentinel-only right now |

## When To Pause And Update Docs

Pause local implementation and update docs when:

- a new product concept or term appears
- a module starts owning a concept outside its boundary
- storage/API compatibility behavior changes
- a lifecycle or recovery state is added
- an AI/calibration/automation feature would mutate data-library state
- a calibration or analysis workflow would update durable named parameter refs,
  setup refs, generated config, or devices without a reviewed proposal/audit
  path
- a calibration chain introduces working refs, health gates, retry semantics,
  or pause/review behavior
- a managed routine introduces desired-state setup/device reconciliation,
  parallel device apply, readback semantics, or partial-failure handling
- draft capabilities or stories would be used as implementation input
  before product-analysis revalidation
- old numbered product story, epic, capability, or future-story IDs would be
  reintroduced or treated as active scope
- a spec or milestone draft introduces product scope, product terminology, or
  architecture decisions that are not already owned by upstream docs or ADRs
- a spec contradicts an accepted ADR

## Review Questions Before Implementation

- Which current story-map slice and capability-map capability does this work
  support?
- Which product terms or concepts does it touch?
- Which future boundary would own each concept?
- What data-library, local runtime/API, Python SDK, Desktop, CLI, and export
  effects exist?
- Which strategic future pressure could this design close off, especially
  parameter state, managed run, calibration/review, setup/device state, run
  manifests, export, or reviewed automation?
- What compatibility checks or migrations are needed?
- What validation proves the behavior?
