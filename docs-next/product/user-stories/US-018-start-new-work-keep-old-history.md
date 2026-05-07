# US-018: Start New Work While Old History Stays Put

## Status

Accepted.

## Primary Epic

EPIC-005: Migration ergonomics and future lab scaling.

## Story

As an experimentalist, I want to start recording new measurements in Fricon
while old LabRAD, QCoDeS, Labber, or folder-based history remains where it is
so that migration does not block new data collection.

## Success Criteria

- The MVP communicates that old history can stay in the old system.
- New Fricon measurements may record source aliases or legacy references.
- Later user-written import scripts can use generic APIs when old data needs to
  move forward.
- Fricon does not require a full historical migration before adoption.

## Not In Scope

- Built-in legacy import as an MVP adoption prerequisite.
- Making reopen/export own old-system history migration.

## Related Capabilities

CAP-001, CAP-011, CAP-026, CAP-030.
