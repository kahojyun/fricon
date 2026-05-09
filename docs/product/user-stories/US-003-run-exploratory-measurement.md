# US-003: Run An Interactive Unmanaged Measurement

## Status

Draft pending product-analysis revalidation.

## Primary Epic

EPIC-002: New measurement logging replacement.

## Story

As a Measurement Run Operator, I want to run an interactive unmanaged
measurement from ordinary Python with minimal boilerplate so that Fricon records
the measurement, produced datasets, optional sample/session context, and honest
provenance.

## Success Criteria

- Measurement creation is explicit but short.
- Sample/session context is optional and visible.
- The normal exploratory path does not require Fricon to manage the Python
  process.
- Dataset writers share measurement lifecycle in the normal path.
- Trace-valued, array-valued, and step-record writers can be used from the
  unmanaged path without switching to managed execution.
- Multiple local measurement writers may be active concurrently.
- A crashed script leaves readable partial data.

## Not In Scope

- Managed runner, task queue, visual sweep builder, or device control.
- Automatic notebook state capture.

## Related Capabilities

CAP-003, CAP-004, CAP-005, CAP-008.
