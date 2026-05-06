# US-008: Reopen Measurement Outputs From Python

## Status

Accepted.

## Primary Epic

EPIC-004: Recovery, annotation, reopen, and export.

## Story

As an experimentalist, I want to reopen a measurement or one of its dataset
artifacts from Python by stable ID so that later analysis does not depend on
remembering storage paths.

## Success Criteria

- Desktop and CLI can show copyable Python reopen snippets.
- Public read APIs can open measurements, dataset artifacts, and partial data
  by stable IDs.
- Reopened data preserves scan schema, units, labels, lifecycle state, and
  partial-data semantics.
- Path-based access is not the documented normal path.

## Not In Scope

- Treating the private storage layout as a public API.
- Importing old legacy-system data before new Fricon outputs can be reopened.

## Related Capabilities / Specs

CAP-010, CAP-026, SPEC-001.
