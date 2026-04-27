---
name: roadmap-alignment
description: Align Fricon product ideas, requirement discussions, and proposed docs changes with the current product route, roadmap, existing proposals, and current implementation state. Use when the user asks to discuss product requirements, decide whether an idea belongs in now/next/later/non-goals, update roadmap or project-intent docs, or check whether a future concept conflicts with current dataset semantics work.
---

# Roadmap Alignment

## Objective

Help a solo maintainer turn product ideas into clear project direction without
confusing future concepts with current behavior.

## Path Resolution

Repository paths are relative to the repository root (`<project_root>`).

## Required Context

Read these first:

- `<project_root>/dev-docs/project-intent.md`
- `<project_root>/dev-docs/roadmap.md`
- `<project_root>/dev-docs/README.md`

When the idea touches current dataset behavior, storage, or semantics, also
read:

- `<project_root>/docs/concepts.md`
- `<project_root>/docs/dataset.md`
- `<project_root>/dev-docs/current-storage-notes.md`
- `<project_root>/dev-docs/dataset-semantic-architecture-proposal.md`
- `<project_root>/dev-docs/dataset-semantic-implementation-plan.md`

For cross-boundary implementation implications, check:

- `<project_root>/dev-docs/maintenance-checklist.md`
- `<project_root>/dev-docs/pr-preflight-checklist.md`
- `<project_root>/dev-docs/adr/README.md`

## Workflow

1. Identify whether the user wants discussion only, documentation edits, issue
   planning, or implementation. Do not implement product ideas during roadmap
   alignment unless explicitly asked.
2. Restate the idea in Fricon terms: user value, affected product pillar, and
   likely user surface.
3. Classify the idea:
   - `current behavior`
   - `now`
   - `next`
   - `after dataset semantics`
   - `future concept`
   - `explicit non-goal`
   - `ADR needed before implementation`
4. Check for conflicts with the dataset-first, Python-led, local-first route.
5. Check whether the idea depends on current dataset semantics proposal work.
6. If docs should change, update the smallest appropriate files:
   - `project-intent.md` for durable product direction and design principles
   - `roadmap.md` for sequencing, future concepts, or core model pressure
   - `README.md` only for index/navigation changes
   - ADRs only for settled, durable, cross-cutting decisions
7. Keep future concepts clearly marked as future. Do not write proposal content
   as current behavior until implementation and current docs have caught up.

## Classification Guidance

- Dataset semantics, resolved interpretation, and chart meaning belong before
  higher-level experiment/workflow/AI automation work.
- Parameter, workflow, device, provenance, and AI concepts should not be hidden
  inside dataset names, incidental metadata, or chart heuristics.
- Workflow definitions are above experiment runs.
- AI integration is a cross-cutting automation layer with review and audit
  boundaries for mutating actions.
- Device management remains later unless a concrete workflow requires minimal
  device identity or configuration snapshots.

## Output Contract

For discussion-only requests, return:

- classification
- why it belongs there
- current docs or proposals it relates to
- open questions
- recommended next action

For documentation-edit requests, return:

- files changed
- what changed
- whether any idea remains future-only or ADR-needed
- whether tests were skipped because the change is docs-only
