# Fricon Docs

## Status

Active documentation workspace; product baseline under revalidation.

## Purpose

`docs/` is the single documentation directory for the v0.2+ reset. It owns
the active product-analysis workspace, domain status, architecture constraints,
ADR, research, user-documentation planning, and AI-agent guidance baseline.

## Design Stance

Fricon v0.2 is a clean reset toward
a local lab data library for scientific measurement work.

The reset keeps useful infrastructure where it fits, but it does not preserve
workspace-first storage, API, IPC, or desktop navigation compatibility when
that compatibility would keep the wrong product model alive.

Product planning in this directory uses initial adoption, strategic follow-on,
and ADR-gated labels. Initial adoption is the first usable migration slice, not
a statement that later product-core capabilities are less important. Do not
treat strategic follow-on priorities as semantic-version labels; compatible
features may still ship on the same compatible release line.

## Workflow Stance

This directory is currently the primary work surface. The product baseline is
being rebuilt from the current greenfield analysis. Keep discussion changes
lightweight. Do not add implementation scaffolding, package locks, generated
artifacts, or release automation until the accepted product baseline and
required downstream architecture or ADR inputs call for them.

## Scale Classification

Fricon is an S2 medium modular system:

- The intended product spans a local runtime, Python SDK, CLI, and Desktop UI.
- Data library, measurement, dataset artifact, storage, API, lifecycle,
  provenance, export, and compatibility concepts cross module boundaries.
- AI-assisted development needs explicit global context, not only local feature
  specs.

## Reading Order

1. `postmortems/v0-lessons.md`
2. `decisions/ADR-001-v02-clean-reset-boundary.md`
3. `product/vision.md`
4. `product/personas.md`
5. `product/product-analysis-progress.md`
6. `product/capability-map.md`
7. `product/story-map.md`
8. `product/python-sdk-ux.md` for the Python SDK usage guideline
9. `product/future-concepts.md` only for strategic follow-on backlog context
10. `domain/README.md`
11. `architecture/README.md`
12. `architecture/compatibility-policy.md`
13. `ai/project-context.md`

`specs/` and `implementation-plans/` are currently sentinels only. Recreate
downstream artifacts only when implementation planning is the task and the
relevant upstream baseline is accepted or explicitly marked with open interview
questions.

## Directory Map

```text
docs/
  product/              Product analysis, user goals, capabilities, stories, glossary
  domain/               Current domain-layer status
  architecture/         Accepted constraints and deferred ADR questions
  decisions/            ADRs for durable decisions
  specs/                Sentinel now; later system-slice specs derived from the baseline
  implementation-plans/ Sentinel now; later milestone plans and quality gates
  postmortems/          Prototype lessons and reset rationale
  research/             Background research process and accepted lessons
  ai/                   Agent context and documentation update policy
  user/                 Future public documentation plan, not current docs
```

## Source-Of-Truth Ownership

Keep each idea in the narrowest durable owner:

- `product/vision.md` owns the current high-confidence product thesis, initial
  adoption goal, user promise, and high-level product horizons.
- `product/personas.md` owns the current high-confidence product-role model.
- `product/product-analysis-progress.md` owns product-analysis progress,
  document confidence, open questions, and the next analysis sequence.
- `product/story-map.md` and `product/capability-map.md` own the current
  draft story/capability baseline derived from the latest high-confidence
  inputs. They intentionally do not preserve older draft IDs; fresh stable IDs
  can be introduced later after the new boundaries are accepted.
- `product/future-concepts.md` is strategic follow-on backlog material until
  the initial adoption product baseline is stable. It preserves future pressure
  without preserving older draft IDs.
- `product/glossary.md` is a provisional terminology helper until capability
  and story analysis settles the active product language.
- `domain/README.md` owns the current domain-layer status.
- `architecture/README.md` owns accepted architecture constraints while the
  project is still in product analysis. Detailed API, storage, module,
  runtime, and export shape remains deferred until later ADRs/specs.
- When active, `specs/` own implementation-slice requirements, design, tasks,
  traceability, and validation derived from upstream sources. They are currently
  sentinels only.
- While product, architecture, or required ADR boundaries are still
  unsettled, `specs/` and `implementation-plans/` must remain sentinel-only:
  do not let them introduce product scope, product terminology, or architecture
  decisions that are not already owned upstream.
- `ai/` owns agent routing and update policy, not product or domain rationale.

Detailed editing rules live in `ai/documentation-update-policy.md`; load that
file only when authoring or reorganizing documentation.

## Token-Efficient Documentation

Future AI sessions should be able to read only the relevant files. Prefer:

- compact bullets over large tables
- slice-level maps over full cross-product matrices
- separate story files only after the product baseline is accepted and an ID
  policy is intentionally chosen
- specs for detailed acceptance criteria and validation
