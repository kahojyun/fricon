# EPIC-005: Migration Ergonomics And Future Lab Scaling

## Status

Accepted.

## Product Goal

Help labs adopt Fricon for new measurements while old loggers, copied folders,
and historical data remain where they are. This epic protects the MVP from
becoming a legacy import project or a full automation stack.

## MVP Scope

- Data Vault-style script migration guidance for new measurements.
- Source aliases and old-system references as metadata, not primary identity.
- Honest code provenance for interactive unmanaged Python.
- Light contextual summaries for setup, environment, procedure, and parameters.
- Run-bound local configuration snapshots or summaries for files and references
  that old scripts currently leave in copied folders.
- Optional sample/session context that can be corrected after a run.

## Post-MVP Scope

- User-written import helpers for old data when generic APIs mature.
- Read-only LAN viewing.
- Measurement-code source setup and approved code update flows.
- Managed code/source provenance for measurement, analysis, and calibration
  evidence, replacing copied working folders as the normal explanation for
  where code came from.
- Rich sample maps, saved views, comparison, and context correction UX.
- Parameter profiles, effective snapshots, diffs, proposals, and reviewed
  durable promotion of calibration-derived settings.
- Calibration records, chains, working refs, health gates, and proposals that
  preserve source measurements, analysis attempts, generated sidecars,
  affected parameter paths, diffs, and rollback targets before durable named
  refs are promoted.
- Managed execution, device communication, and AI-assisted reviewed automation
  after the code/parameter evidence model is durable.

## Key Stories

- US-011: Record code provenance honestly.
- US-012: Register a sample and sample session when useful.
- US-014: Record passive setup context.
- US-016: Record passive procedure context.
- US-017: Migrate a Data Vault-style script to Fricon writers.
- US-018: Start new work in Fricon while old history stays in the old system.
- US-019: Capture run-bound local configuration.
