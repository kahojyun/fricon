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

## Scale Classification

Fricon is an S2 medium modular system:

- Rust core/service code, Python SDK, CLI, and Tauri/React desktop UI interact.
- Data library, measurement, dataset artifact, storage, API, lifecycle,
  provenance, export, and compatibility concepts cross module boundaries.
- AI-assisted development needs explicit global context, not only local feature
  specs.

## Reading Order

1. `redesign-from-prototype.md`
2. `postmortems/v0-lessons.md`
3. `decisions/ADR-001-v02-clean-reset-boundary.md`
4. `product/vision.md`
5. `product/capability-map.md`
6. `product/story-map.md`
7. `product/python-sdk-ux.md` for the Python SDK usage guideline
8. relevant `product/epics/` and `product/user-stories/`
9. `domain/conceptual-model.md`
10. `architecture/system-overview.md`
11. `architecture/module-boundaries.md`
12. `architecture/compatibility-policy.md`
13. `ai/project-context.md`
14. relevant `specs/`

## Directory Map

```text
docs-next/
  redesign-from-prototype.md
  product/              User goals, capabilities, stories, future ledger, glossary
  domain/               Conceptual model, contexts, lifecycles, invariants
  architecture/         System shape, boundaries, data flow, storage, API
  decisions/            ADRs for durable decisions
  specs/                System-slice specs derived from the baseline
  implementation-plans/ Milestone plans and quality gates
  postmortems/          Prototype lessons and reset rationale
  research/             Background research process and accepted lessons
  ai/                   Agent context and documentation update policy
  user/                 Future public documentation plan, not current docs
```

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
