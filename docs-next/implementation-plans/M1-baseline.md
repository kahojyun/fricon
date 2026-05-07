# M1: v0.2 Foundation Baseline

## Status

Draft downstream shell. Not implementation-ready.

## Planning Boundary

This file is a milestone landing area, not a source of product or domain scope.
Until the product, domain, architecture, and required ADR boundaries are
accepted, upstream documents own vocabulary, scope, non-goals, and sequencing.

Do not add capability lists, domain records, API/storage choices, validation
scenarios, or work sequences here to answer unsettled upstream questions. If a
milestone question reveals missing scope or vocabulary, interview for the
decision and update the upstream owner first.

## Goal

Track readiness for the first v0.2 implementation milestone after it can be
derived from accepted upstream material.

## Upstream Inputs

- `product/vision.md`
- `product/capability-map.md`
- `product/story-map.md`
- relevant `product/user-stories/`
- `product/python-sdk-ux.md`
- `domain/conceptual-model.md`
- `domain/context-map.md`
- `domain/lifecycle-model.md`
- `domain/invariants.md`
- `architecture/system-overview.md`
- `architecture/module-boundaries.md`
- `architecture/storage-model.md`
- `architecture/api-boundaries.md`
- `architecture/compatibility-policy.md`
- accepted ADRs

## Readiness Gate

M1 is ready to plan when:

- upstream product and domain scope is accepted or explicitly marked with open
  interview questions
- required architecture and compatibility ADRs are accepted
- a derived spec has accepted requirements, design, traceability, and validation
- implementation tasks are generated from that accepted spec, not from this file
- migration/import stance for pre-v0.2 data is explicit upstream
