# US-018: Start New Work While Old History Stays Put

## Status

Draft.

## Story

As an experimentalist, I want to start recording new measurements in Fricon
while old LabRAD, QCoDeS, Labber, or folder-based history remains where it is
so that migration does not block new data collection.

## Acceptance Notes

- v0.2 communicates that old history can stay in the old system.
- New Fricon measurements may record source aliases or legacy references.
- Later user-written import scripts can use generic APIs when old data needs to
  move forward.
- Fricon does not require a full historical migration before adoption.

## Related

CAP-001, CAP-011, CAP-026, CAP-030.
