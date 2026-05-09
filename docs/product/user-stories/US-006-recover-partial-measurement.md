# US-006: Recover A Partial Measurement

## Status

Draft pending product-analysis revalidation.

## Primary Epic

EPIC-004: Recovery, annotation, reopen, and export.

## Story

As a Measurement Run Operator, I want interrupted or failed measurements to
remain readable so that a crash does not turn useful partial data into an
opaque folder cleanup problem.

## Success Criteria

- Measurement lifecycle shows interrupted, failed, aborted, invalidated,
  superseded, recovered, or completed state clearly.
- Partial dataset artifacts can be reopened and inspected through public read
  APIs.
- Written trace records, IQ arrays, and irregular step records remain readable
  when the script stops before the measurement is completed.
- Missing expected points are represented when the scan schema supports that
  interpretation.
- Rerun creates a new linked measurement by default.
- Appending to an older measurement requires explicit resume intent and
  compatibility checks.
- Ordinary cleanup uses trash/recover instead of immediate hard delete.

## Not In Scope

- Resuming unmanaged Python execution from the last scan point.
- Hiding incomplete state to make partial data look complete.

## Related Capabilities

CAP-008, CAP-010.
