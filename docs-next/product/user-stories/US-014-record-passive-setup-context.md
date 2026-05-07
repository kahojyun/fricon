# US-014: Record Passive Setup Context

## Status

Accepted.

## Primary Epic

EPIC-005: Migration ergonomics and future lab scaling.

## Story

As an experimentalist, I want to record light setup, device, method, software,
or environment summaries for a measurement so that future analysis has
practical context without requiring Fricon to control instruments.

## Success Criteria

- Measurement context can include optional setup, device, method, software, or
  environment labels and summaries.
- Context may be user-supplied, imported from an external reference, or
  corrected after the run.
- Structured passive context can include runner labels, device-map labels,
  wiring/config references, software environment notes, machine labels, and
  selected network or address summaries when the user chooses to record them.
- Setup context can link to a run-bound local configuration summary without
  making that summary a managed hardware inventory.
- Export previews sensitive local paths, machine names, or environment details.
- Passive setup context is clearly separate from device control.

## Not In Scope

- Device communication, calibration registry, or managed hardware inventory.
- Claiming setup context is complete when it is only a passive summary.

## Related Capabilities

CAP-027.
