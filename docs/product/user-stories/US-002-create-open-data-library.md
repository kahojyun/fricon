# US-002: Create Or Open One Local Data Library

## Status

Accepted.

## Primary Epic

EPIC-001: Local setup and data-library adoption.

## Story

As a Local Measurement System Maintainer, I want to create or open one local
Fricon data library for a lab computer so that measurements, datasets, notes,
and context have a durable home.

## Success Criteria

- A new library gets a durable identity, display name, remembered location, and
  format version.
- Opening an existing library verifies identity and compatibility before any
  mutation.
- Recent-library selection is clear enough to avoid writing into the wrong
  place.
- Locked, unsupported, or migration-required libraries produce actionable
  diagnostics.

## Not In Scope

- Multiple writers directly editing the same shared-folder database.
- Hosted or account-based library ownership.

## Related Capabilities

CAP-001, CAP-012, CAP-013.
