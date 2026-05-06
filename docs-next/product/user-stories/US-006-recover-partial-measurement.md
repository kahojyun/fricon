# US-006: Recover A Partial Measurement

## Status

Accepted.

## Primary Epic

EPIC-004: Recovery, annotation, reopen, and export.

## Story

As an experimentalist, I want interrupted or failed measurements to remain
readable so that a crash does not turn useful partial data into an opaque
folder cleanup problem.

## Success Criteria

- Measurement lifecycle shows interrupted, failed, invalidated, recovered, or
  completed state clearly.
- Partial dataset artifacts can be reopened and inspected through public read
  APIs.
- Missing expected points are represented when the scan schema supports that
  interpretation.
- Ordinary cleanup uses trash/recover instead of immediate hard delete.

## Not In Scope

- Resuming unmanaged Python execution from the last scan point.
- Hiding incomplete state to make partial data look complete.

## Related Capabilities / Specs

CAP-008, CAP-010, SPEC-001.
