# US-004: Produce Dataset Artifacts

## Status

Draft pending product-analysis revalidation.

## Primary Epic

EPIC-002: New measurement logging replacement.

## Story

As a Measurement Run Operator, I want one measurement to produce one or more
dataset artifacts so that related scans, step records, arrays, or traces stay
connected without losing direct dataset access.

## Success Criteria

- A measurement can create multiple dataset artifacts with stable IDs.
- Dataset artifacts carry scan, step-record, array, or trace schema where
  plotting and slicing semantics matter.
- Trace-valued records can carry explicit coordinate/value arrays or compact
  regular-coordinate descriptions, and a single outer sweep record can contain
  more than one trace.
- Complex values are preserved as first-class data so magnitude/phase or I/Q
  views can be derived from data semantics instead of channel-name guesses.
- Dataset writers share measurement lifecycle by default while remaining
  individually discoverable.
- Dataset facts are appendable while a writer is active and immutable after the
  dataset artifact is completed.
- Corrections to completed dataset meaning are recorded as events or derived
  artifacts rather than silent fact edits.
- Internal streams or subpayloads can exist below the user concept when needed
  for storage, export, or reading.

## Not In Scope

- Making internal streams the normal user-facing product model.
- Requiring one measurement to have exactly one dataset.

## Related Capabilities

CAP-005, CAP-006, CAP-026, CAP-028.
