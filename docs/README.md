# Fricon Docs

## Status

Active v0.2+ documentation baseline.

## Purpose

`docs/` is the single documentation directory for the v0.2+ reset. It owns
the active product, domain, architecture, ADR, research, user-documentation
planning, and AI-agent guidance baseline.

## Design Stance

Fricon v0.2 is a clean reset toward
a local lab data library for scientific measurement work.

The reset keeps useful infrastructure where it fits, but it does not preserve
workspace-first storage, API, IPC, or desktop navigation compatibility when
that compatibility would keep the wrong product model alive.

Product planning in this directory uses MVP, post-MVP priority, and ADR-gated
labels. Do not treat post-MVP priorities as semantic-version labels; compatible
features may still ship on the same compatible release line.

## Workflow Stance

This directory is currently the primary work surface. Keep discussion changes
lightweight. Do not add implementation scaffolding, package locks, generated
artifacts, or release automation until the accepted product, domain,
architecture, and ADR baseline calls for them.

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
4. `product/capability-map.md`
5. `product/story-map.md`
6. `product/python-sdk-ux.md` for the Python SDK usage guideline
7. relevant `product/epics/` and `product/user-stories/`
8. `domain/conceptual-model.md`
9. `architecture/README.md`
10. `architecture/compatibility-policy.md`
11. `ai/project-context.md`

`specs/` and `implementation-plans/` are currently sentinels only. Recreate
downstream artifacts only when implementation planning is the task and the
relevant upstream baseline is accepted or explicitly marked with open interview
questions.

## Directory Map

```text
docs/
  product/              User goals, capabilities, stories, future ledger, glossary
  domain/               Conceptual model, contexts, lifecycles, invariants
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

- `product/vision.md` owns the accepted product thesis, MVP goal, user promise,
  and high-level product horizons.
- `product/future-concepts.md` owns the accepted post-MVP priority ledger and
  promotion rule.
- `product/future-stories-and-requirements.md` owns proposed future user stories
  and requirements. It should reference the priority ledger instead of restating
  its rationale.
- `product/capability-map.md` owns stable capability IDs and compact scope
  boundaries. Detailed acceptance notes belong in stories or ADRs; derived
  specs may add implementation acceptance after upstream acceptance.
- `product/glossary.md` owns public and future terminology.
- `domain/conceptual-model.md`, `domain/context-map.md`, and
  `domain/invariants.md` own concept relationships, bounded-context routing,
  and hard anti-corruption rules.
- `architecture/README.md` owns accepted architecture constraints while the
  project is still in product/domain analysis. Detailed API, storage, module,
  runtime, and export shape remains deferred until later ADRs/specs.
- When active, `specs/` own implementation-slice requirements, design, tasks,
  traceability, and validation derived from upstream sources. They are currently
  sentinels only.
- While product, domain, architecture, or required ADR boundaries are still
  unsettled, `specs/` and `implementation-plans/` must remain sentinel-only:
  do not let them introduce product scope, domain vocabulary, or architecture
  decisions that are not already owned upstream.
- `ai/` owns agent routing and update policy, not product or domain rationale.

Detailed editing rules live in `ai/documentation-update-policy.md`; load that
file only when authoring or reorganizing documentation.

## Token-Efficient Documentation

Future AI sessions should be able to read only the relevant files. Prefer:

- compact bullets over large tables
- epic-level maps over full cross-product matrices
- separate user-story files for high-value stories only
- specs for detailed acceptance criteria and validation
