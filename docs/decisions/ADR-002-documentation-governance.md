# ADR-002: Documentation Governance For v0.2+

## Status

Accepted.

## Context

Fricon is an architecture-sensitive system with Rust core/service code, Python
bindings, CLI, desktop UI, local storage, protocol boundaries, and future
automation surfaces. Feature-local specs alone are not enough to preserve the
domain model.

## Decision

Use the documentation tree as the v0.2+ baseline with these artifact classes:

- product: vision, personas, capability map, story map, glossary, traceability
- domain: conceptual model, context map, lifecycle, invariants
- architecture: overview, module boundaries, data flow, API/storage,
  compatibility, risks
- decisions: ADRs for durable decisions
- specs: system-slice specs derived from the global baseline
- implementation plans: milestone execution and quality gates
- postmortems: reset lessons
- ai: project context, steering, and doc update policy
- user: future public documentation plan

Documents use explicit status values: Draft, Proposed, Accepted,
Implementing, Implemented, Superseded, Deprecated.

Important IDs are stable:

- CAP-### for capabilities
- US-### for user stories
- SPEC-### for specs
- REQ-### inside specs
- ADR-### for decisions
- M# for milestones

Detailed authoring and ID-allocation workflow is maintained in
`ai/documentation-update-policy.md`.

## Consequences

- Do not implement major v0.2 features from chat history or archived proposals
  alone.
- New domain vocabulary must update domain docs before or with implementation.
- Specs must trace to capabilities, stories, domain concepts, architecture
  docs, ADRs, modules, tests, compatibility, and non-goals.
- Superseded planning material should be folded into the owning product,
  domain, ADR, or user-documentation plan, then removed.

## Alternatives Considered

- Keep only a roadmap and issue list. This is too weak for cross-module
  semantics.
- Put all design detail in ADRs. ADRs are good for decisions but poor as the
  full product/domain model.
- Put all design detail in implementation specs. Specs become too local and
  make global concepts hard to find.

## Revisit Triggers

- The documentation baseline becomes too large to maintain.
- The project moves from v0.x exploration to stable long-term compatibility.
- A new contributor workflow proves that the reading order is inefficient.
