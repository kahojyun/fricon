# Fricon Docs Next

## Status

Draft v0.2+ documentation baseline.

## Purpose

`docs-next/` is the clean design workspace for the v0.2+ reset. It exists so
new product, domain, architecture, and implementation planning can move forward
without mixing:

- current v0.1 implementation facts
- historical proposal notes
- AI-agent process discussion
- new v0.2+ design decisions

For v0.2+ planning, treat this directory as the starting point.
`dev-docs/project-intent.md` remains strategic product canon; `docs-next/`
derives from it and owns the v0.2+ design baseline. Use older `dev-docs/v0.2/`
proposal files as source material and historical rationale, not as the current
source of truth.

## Design Stance

Fricon v0.2 is a clean reset from the workspace/dataset-first prototype toward
a local lab data library for scientific measurement work.

The reset keeps useful infrastructure where it fits, but it does not preserve
v0.1 workspace, storage, API, IPC, or desktop navigation compatibility when
that compatibility would keep the wrong user model alive.

Product planning in this directory uses MVP, post-MVP priority, and ADR-gated
labels. Do not treat post-MVP priorities as semantic-version labels; compatible
features may still ship on the same compatible release line.

## Workflow Stance

This directory is currently the primary work surface. Keep discussion changes
lightweight: do not run full Rust, Python, frontend, release, or docs-deploy
automation for `docs-next/`-only edits unless a task explicitly asks for it.
Repository automation such as GitHub Actions, Renovate, release publishing,
and docs deployment is intentionally paused while the v0.2+ baseline is being
discussed. Basic text and config hygiene remains available through
`.pre-commit-config.yaml`, but it should not run dependency updates,
generated-artifact checks, language builds, or test gates.

## Scale Classification

Fricon is an S2 medium modular system:

- Rust core/service code, Python SDK, CLI, and Tauri/React desktop UI interact.
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
docs-next/
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

## Source Inputs

This baseline was bootstrapped from:

- `README.md`
- `docs/`
- `dev-docs/project-intent.md`
- `dev-docs/roadmap.md`
- `dev-docs/current-storage-notes.md`
- `dev-docs/current-dataset-semantics.md`
- `dev-docs/architecture-guidelines.md`
- `dev-docs/v0.2/`
- `dev-docs/v0.2/archive/`

If an old proposal conflicts with this baseline, prefer this baseline for v0.2+
planning unless a newer ADR says otherwise.

## Token-Efficient Documentation

Future AI sessions should be able to read only the relevant files. Prefer:

- compact bullets over large tables
- epic-level maps over full cross-product matrices
- separate user-story files for high-value stories only
- specs for detailed acceptance criteria and validation
