# US-016: Record Passive Procedure Context

## Status

Accepted.

## Primary Epic

EPIC-005: Migration ergonomics and future lab scaling.

## Story

As an experimentalist, I want to record an unmanaged script, external runner,
or declared procedure summary so that the measurement has procedural context
even before Fricon manages execution.

## Success Criteria

- A measurement can link an unmanaged script summary, external-runner
  reference, or declared plan summary.
- Procedure context can mention planned shape, relevant parameters, and
  external identifiers without becoming a managed plan.
- Corrections to procedure context are visible after the run.
- Procedure context remains distinct from code provenance and setup context.

## Not In Scope

- Managed measurement plans, task queues, or scan-point checkpoints.
- Claiming Fricon executed steps it only observed or recorded.

## Related Capabilities / Specs

CAP-029, SPEC-001.
