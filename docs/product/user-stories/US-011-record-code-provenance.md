# US-011: Record Code Provenance Honestly

## Status

Draft pending product-analysis revalidation.

## Primary Epic

EPIC-005: Migration ergonomics and future lab scaling.

## Story

As a Measurement Method Author, I want Fricon to record what it can honestly
know about the code that produced a measurement so that later analysis can
judge trustworthiness without pretending Fricon managed execution.

## Success Criteria

- Unmanaged Python is labeled as unmanaged unless Fricon actually controls
  execution.
- Optional script path, copied-folder/source label, user summary, and Git
  revision only where meaningful can be recorded where available.
- Copied-folder workflows can record a source-root path, folder fingerprint, or
  user-supplied source label without implying managed code history.
- Code provenance is distinct from passive procedure context and run-bound
  local configuration: code provenance explains where the Python/user code came
  from, procedure context explains what was intended or invoked, and run config
  explains selected files or settings that were active.
- The product model leaves room for approved code releases, setup profiles,
  environment lock files, scan helpers, plot presets, export recipes, and
  handoff to a Local Measurement System Maintainer without making them initial
  adoption code-management features.
- Provenance can be previewed or omitted during export when it contains
  sensitive local details.
- Corrections to provenance are visible as events.

## Not In Scope

- Automatic notebook state capture.
- Managed code snapshots, deployment, or approved update flows.

## Related Capabilities

CAP-014, CAP-017.
