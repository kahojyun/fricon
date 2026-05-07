# US-010: Update Without Corrupting Measurement Work

## Status

Accepted.

## Primary Epic

EPIC-001: Local setup and data-library adoption.

## Story

As an experimentalist, I want Fricon updates and format changes to fail safely
so that measurement work is not corrupted by mismatched Desktop, service, CLI,
Python SDK, or library versions.

## Success Criteria

- Mutating clients negotiate compatibility with the service and library before
  writes.
- Incompatible versions fail before mutation and explain the required action.
- Format migrations or repair operations create or require a checkpoint
  strategy.
- Users can tell whether a problem is service state, library state, client
  version, or data-format compatibility.

## Not In Scope

- Stable third-party protocol commitments before the MVP is proven.
- Automatic migration of old v0.1 workspaces or legacy systems.

## Related Capabilities

CAP-012, CAP-013.
