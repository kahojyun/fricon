---
name: issue-planning
description: Convert already-aligned Fricon roadmap items or milestones into small GitHub-ready issues or task plans for low-conflict Codex work. Use only when the user explicitly asks to create an issue plan, decompose roadmap work into tasks, align roadmap with issues/tasks, prepare parallel PR tasks, or create GitHub issues. Do not trigger for product brainstorming, roadmap editing, ordinary implementation, or PR preflight.
---

# Issue Planning

## Objective

Act as the missing lightweight project-manager layer for a solo Fricon
maintainer. Turn roadmap direction into small, reviewable, low-conflict tasks
without starting implementation.

## Trigger Boundaries

Use this skill only after the product direction is already clear enough to plan
tasks. If the user is still deciding what the product should do or where an
idea belongs in the roadmap, use `roadmap-alignment` first. If the user asks to
implement issue `#xxx`, do not use this skill unless they also ask to re-plan
or split that issue.

## Path Resolution

Repository paths are relative to the repository root (`<project_root>`).

## Required Context

Read these first:

- `<project_root>/dev-docs/project-intent.md`
- `<project_root>/dev-docs/roadmap.md`
- `<project_root>/dev-docs/README.md`
- `<project_root>/dev-docs/pr-preflight-checklist.md`
- `<project_root>/dev-docs/maintenance-checklist.md`

When planning dataset semantics work, also read:

- `<project_root>/dev-docs/current-storage-notes.md`
- `<project_root>/dev-docs/dataset-semantic-architecture-proposal.md`
- `<project_root>/dev-docs/dataset-semantic-implementation-plan.md`

When planning a UI-heavy milestone, also read:

- `<project_root>/dev-docs/desktop-ui-feature-playbook.md`
- `<project_root>/dev-docs/testing-strategy.md`

## Workflow

1. Confirm the planning target: roadmap area, milestone, product idea, or issue
   theme. If the target is not aligned with `roadmap.md` yet, stop and use
   `roadmap-alignment` first.
2. Identify dependencies and whether any ADR or design note is needed before
   implementation.
3. Split work into issues that are each suitable for one focused pull request.
4. Prefer dependency order that preserves the current route:
   dataset semantics -> inspection/Python usability -> run records/parameters
   -> provenance -> workflows/scheduler -> AI automation -> device integration.
5. Mark each issue as one of:
   - `backend`
   - `Python API`
   - `desktop UI`
   - `docs`
   - `tests`
   - `cross-boundary`
   - `ADR/design`
6. For each issue, list:
   - goal
   - scope
   - out of scope
   - likely touched files or modules
   - dependencies or blockers
   - parallelization risk: `low`, `medium`, or `high`
   - validation commands or checklist sections
   - expected PR shape
7. Keep issues small. Split when an issue crosses unrelated feature slices,
   requires both design and implementation, or would block several parallel
   Codex threads.
8. If the user asks to create GitHub issues, use the GitHub connector when
   available. Otherwise produce a markdown issue plan that can be reviewed
   before creation.

## Parallel Work Guidance

- Low-risk parallel issues should touch distinct files or feature slices.
- High-conflict files include central docs, generated bindings, database schema,
  migration files, IPC/gRPC contracts, and shared frontend app shells.
- Cross-boundary changes need explicit checklist references and should usually
  be sequenced before dependent UI or Python API issues.
- If two tasks need the same high-conflict file, make one a blocking foundation
  issue or split out a shared preparatory issue.

## Issue Template

```markdown
## Goal

## Scope

## Out Of Scope

## Likely Files Or Modules

## Dependencies

## Parallelization Risk

## Validation

## PR Shape
```

## Output Contract

Return:

- milestone or planning theme
- ordered issue list
- dependency notes
- parallelization recommendations
- ADR/design tasks that should happen before implementation
- issues created, if GitHub issue creation was requested and completed
