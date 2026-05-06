# Lifecycle Model

## Status

Draft.

## Data Library Lifecycle

```text
created
  -> opened
  -> idle
  -> active
  -> draining
  -> migrating
  -> idle
  -> backed_up/restored
```

Rules:

- Migrations require idle or draining state with no active writers.
- Backup/restore is a user-visible safety path.
- Incompatible clients fail before writes.

## Measurement Lifecycle

```text
draft/created
  -> active
  -> finishing
  -> finished

active
  -> interrupted
  -> failed
  -> aborted

finished | interrupted | failed | aborted
  -> trashed
  -> recovered

finished | interrupted | failed
  -> invalidated
  -> superseded
```

Rules:

- Partial facts remain visible after interruption or failure.
- Rerun creates a new linked measurement by default.
- Appending to an older measurement requires explicit resume intent and
  compatibility checks.
- Cleanup uses trash/recover before hard delete.

## Dataset Artifact Lifecycle

```text
declared
  -> writing
  -> finishing
  -> complete

writing
  -> interrupted
  -> aborted

complete | interrupted
  -> exported
  -> consumed_by_analysis_later
```

Rules:

- Facts are appendable while writing.
- Facts are immutable after complete.
- Corrections create events or derived artifacts rather than silent fact edits.
- Live readers tail explicit append positions and may drop preview updates, but
  committed data is never dropped.

## Event Timeline

Measurement timelines combine:

- lifecycle events
- notes and markers
- context corrections
- provenance updates
- system events
- export or recovery actions

Events should carry an actor label when practical.
