# Documentation Update Policy

## Status

Accepted.

## Before Implementation

For v0.2+ system work:

- read the relevant product, domain, architecture, and ADR documents
- confirm the spec status is Accepted or the user explicitly permits Draft work
- update docs first if implementation would contradict accepted direction
- record non-goals and compatibility impact in the spec

## During Implementation

Update docs in the same change when:

- a public concept, field, lifecycle state, or capability changes
- a module boundary changes
- a compatibility or migration rule changes
- a validation requirement changes

## After Implementation

Gate D documentation sync requires:

- spec tasks reflect actual status
- traceability lists affected modules/files at the right granularity
- public docs are updated if implemented behavior is user-visible
- architecture docs or ADRs reflect significant deviations
- stale docs are listed with next actions if immediate update is not possible

## Staleness Policy

Mark stale documents explicitly:

- `Superseded` when replaced by a newer source of truth
- `Deprecated` when retained for history but not guiding new work
- `Draft` when exploratory

Do not silently delete old planning material unless the user asks for cleanup.
