# US-011: Record Code Provenance Honestly

## Status

Draft.

## Primary Epic

EPIC-005: Migration ergonomics and future lab scaling.

## Story

As an experimentalist, I want Fricon to record what it can honestly know about
the code that produced a measurement so that later analysis can judge
trustworthiness without pretending Fricon managed execution.

## Success Criteria

- Unmanaged Python is labeled as unmanaged unless Fricon actually controls
  execution.
- Optional script path, Git revision, dirty-state signal, and user summary can
  be recorded where available.
- Provenance can be previewed or omitted during export when it contains
  sensitive local details.
- Corrections to provenance are visible as events.

## Not In Scope

- Automatic notebook state capture.
- Managed code snapshots, deployment, or approved update flows.

## Related Capabilities / Specs

CAP-014, CAP-017, SPEC-001.
