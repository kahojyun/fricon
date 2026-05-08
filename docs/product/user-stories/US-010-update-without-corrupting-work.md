# US-010: Update Without Corrupting Measurement Work

## Status

Accepted.

## Primary Epic

EPIC-001: Local setup and data-library adoption.

## Story

As a Local Measurement System Maintainer, I want Fricon updates and format
changes to fail safely so that measurement work is not corrupted by mismatched
Desktop, local runtime, CLI, Python SDK, or library versions.

## Success Criteria

- Mutating clients negotiate compatibility with the local Fricon runtime and
  library before writes.
- Incompatible versions fail before mutation and explain the required action.
- Format migrations or repair operations create or require a checkpoint
  strategy.
- Users can tell whether a problem is local runtime state, library state,
  client version, or data-format compatibility.
- Updates or migrations do not interrupt active measurements, open writers,
  imports, exports, or repair work without explicit user intent.
- Users can defer update work until the local measurement environment is idle.
- Locked-down lab-computer needs such as offline installers, rollback, or
  side-by-side installs remain product pressure, not MVP polish.

## Not In Scope

- Stable third-party protocol commitments before the MVP is proven.
- Automatic migration of old v0.1 workspaces or legacy systems.
- Silent auto-update while measurement work is active.

## Related Capabilities

CAP-012, CAP-013.
