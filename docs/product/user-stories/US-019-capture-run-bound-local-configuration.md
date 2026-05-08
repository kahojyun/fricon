# US-019: Capture Run-Bound Local Configuration

## Status

Accepted.

## Primary Epic

EPIC-005: Migration ergonomics and future lab scaling.

## Story

As an Effective Configuration Steward, I want to bind selected local
configuration files, references, hashes, or summaries to a measurement so that
later analysis can explain which lab-local state was active without reading
copied folders by hand.

## Success Criteria

- A measurement can attach a run-bound configuration summary at creation time
  or during the run.
- The summary can reference or snapshot selected files such as
  `parameters.json`, `registry.json`, wiring sheets, line/chip info, demod
  settings, readout settings, or external runner configuration.
- Fricon records enough source identity to distinguish a copied file reference,
  a copied file snapshot, a content hash, and a user-entered summary.
- Corrections to configuration context are visible as events.
- Export previews sensitive local paths, machine names, IP addresses, and other
  local setup details before including them.

## Not In Scope

- Automatic tracing of every file a script reads.
- Global parameter profiles, proposal workflows, or calibration promotion.
- Device control, hardware inventory, or claiming unmanaged execution is fully
  reproducible.

## Related Capabilities

CAP-015, CAP-027, CAP-029, CAP-033.
