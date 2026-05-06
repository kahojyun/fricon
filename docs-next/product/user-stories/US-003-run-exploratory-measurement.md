# US-003: Run An Exploratory Measurement

## Status

Draft.

## Primary Epic

EPIC-002: New measurement logging replacement.

## Story

As an experimentalist, I want to run a measurement from Python with minimal
boilerplate so that Fricon records the measurement, produced datasets, optional
sample/session context, and basic provenance.

## Success Criteria

- Measurement creation is explicit but short.
- Sample/session context is optional and visible.
- Dataset writers share measurement lifecycle in the normal path.
- Multiple local measurement writers may be active concurrently.
- A crashed script leaves readable partial data.

## Not In Scope

- Managed runner, task queue, visual sweep builder, or device control.
- Automatic notebook state capture.

## Related Capabilities / Specs

CAP-003, CAP-004, CAP-005, CAP-008, SPEC-001.
