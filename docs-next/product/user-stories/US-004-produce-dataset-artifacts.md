# US-004: Produce Dataset Artifacts

## Status

Accepted.

## Primary Epic

EPIC-002: New measurement logging replacement.

## Story

As an experimentalist, I want one measurement to produce one or more dataset
artifacts so that related tables, scans, or traces stay connected without
losing direct dataset access.

## Success Criteria

- A measurement can create multiple dataset artifacts with stable IDs.
- Dataset artifacts carry scan or trace schema where plotting semantics matter.
- Dataset writers share measurement lifecycle by default while remaining
  individually discoverable.
- Internal streams or subpayloads can exist below the user concept when needed
  for storage, export, or reading.

## Not In Scope

- Making internal streams the normal user-facing product model.
- Requiring one measurement to have exactly one dataset.

## Related Capabilities / Specs

CAP-005, CAP-006, CAP-026, CAP-028, SPEC-001.
