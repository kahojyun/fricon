# Specs

## Status

Sentinel. No active implementation specs.

## Rule

Do not create or use system-slice specs while product, domain, architecture, or
required ADR boundaries are still unsettled.

Specs may be recreated only after the upstream owner documents are accepted or
explicitly marked with open interview questions. A recreated spec must be
derived from upstream sources and must not introduce product scope, product or
domain terminology, architecture decisions, or non-goals on its own.

## Recreation Criteria

Before adding `SPEC-###-*`, confirm:

- related capability and story IDs are accepted upstream
- related product concepts, domain concepts, and invariants are accepted
  upstream
- storage, API, compatibility, and module-boundary decisions are accepted or
  tracked as ADR questions
- any unresolved decision has been handled by interview and recorded upstream
