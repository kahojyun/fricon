# US-014: Record Passive Setup Context

## Status

Draft.

## Primary Epic

EPIC-005: Migration ergonomics and future lab scaling.

## Story

As an experimentalist, I want to record setup, device, method, software, or
environment summaries for a measurement so that future analysis has practical
context without requiring Fricon to control instruments.

## Success Criteria

- Measurement context can include optional setup, device, method, software, or
  environment labels and summaries.
- Context may be user-supplied, inferred, imported from an external reference,
  or corrected after the run.
- Export previews sensitive local paths, machine names, or environment details.
- Passive setup context is clearly separate from device control.

## Not In Scope

- Device communication, calibration registry, or managed hardware inventory.
- Claiming setup context is complete when it is only a passive summary.

## Related Capabilities / Specs

CAP-027, SPEC-001.
