# US-003: Run An Exploratory Measurement

## Status

Draft.

## Story

As an experimentalist, I want to run a measurement from Python with minimal
boilerplate so that Fricon records the measurement, produced datasets, optional
sample/session context, and basic provenance.

## Acceptance Notes

- Measurement creation is explicit but short.
- Sample/session context is optional and visible.
- Dataset writers share measurement lifecycle in the normal path.
- Multiple local measurement writers may be active concurrently.
- A crashed script leaves readable partial data.

## Related

CAP-003, CAP-004, CAP-005, CAP-008.
