# Architecture Decision Records

## Status

Index and rules for durable architecture decision records.

## Purpose

Use ADRs to preserve important decisions after the project has chosen a
direction. ADRs should explain why a decision was made, what alternatives were
considered, and what consequences future work must respect.

ADRs are not task plans, meeting notes, or design brainstorms. Use
`dev-docs/roadmap.md` for coarse direction and focused proposal or
implementation-plan files for unresolved work.

## When To Add An ADR

Add an ADR when a decision is:

- durable enough to guide future changes
- cross-cutting across crates, APIs, storage, IPC, or UI/runtime boundaries
- likely to be rediscovered by future human or AI contributors
- important enough that reversing it would require coordinated migration work

Do not add an ADR for:

- routine implementation details
- temporary workarounds
- narrow refactors contained in one feature slice
- proposals that are still exploratory

## Naming

Use a monotonically increasing four-digit prefix:

```text
0001-record-architecture-decisions.md
0002-example-decision-title.md
```

Use lowercase kebab-case after the number.

## ADR Format

Each ADR should use this structure:

```markdown
# ADR 0000: Decision Title

## Status

Accepted | Superseded by ADR 0000 | Deprecated

## Context

What problem, constraint, or recurring question forced the decision?

## Decision

What has the project decided?

## Consequences

What becomes easier, harder, required, or intentionally unsupported?

## Alternatives Considered

What plausible options were rejected, and why?
```

Keep ADRs concise. Link to deeper design notes or implementation plans instead
of copying their full contents.

## Index

- `0001-record-architecture-decisions.md` - establishes when and how this
  project records ADRs
