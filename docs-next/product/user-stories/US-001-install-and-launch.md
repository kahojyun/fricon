# US-001: Install And Launch Fricon

## Status

Draft.

## Primary Epic

EPIC-001: Local setup and data-library adoption.

## Story

As an experimentalist, I want one clear way to install and launch Fricon on a
lab computer so that I can open Fricon Desktop and use the Python SDK against a
local data library without assembling incompatible pieces by hand.

## Success Criteria

- Fricon Desktop, bundled service, bundled CLI, and Python SDK compatibility are
  visible as one product set.
- First-run setup asks for the data-library location.
- Python scripts can run headlessly when Desktop is closed.
- Diagnostics explain stopped service, wrong library, locked library, old SDK,
  incompatible service, or migration-required states.

## Not In Scope

- Hosted service, multi-user administration, or direct shared-folder database
  editing.
- Enterprise deployment polish before the local loop works.

## Related Capabilities / Specs

CAP-001, CAP-002, CAP-013, SPEC-001.
