# US-012: Register A Sample And Sample Session When Useful

## Status

Accepted.

## Primary Epic

EPIC-005: Migration ergonomics and future lab scaling.

## Story

As an experimentalist, I want to attach optional sample or sample-session
context to a measurement so that data is easier to interpret when sample
identity matters, without blocking exploratory measurements.

## Success Criteria

- Measurement creation can use no sample context, an active sample, or an
  active sample session.
- The chosen context is visible when the measurement starts and can be corrected
  later.
- Dataset and export views carry enough context to answer which sample/session
  was active.
- Missing sample context remains an explicit absence, not a fake sample.

## Not In Scope

- Requiring a complete sample registry before recording data.
- Rich sample maps, saved views, or spatial comparison workflows.

## Related Capabilities

CAP-004.
