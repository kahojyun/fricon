# EPIC-005: Migration Ergonomics And Future Lab Scaling

## Status

Draft.

## Product Goal

Help labs move from copied folders, old loggers, and ad hoc parameter files
toward a local data-library workflow without forcing a hosted service or a full
automation stack on day one.

## v0.2 Scope

- Generic APIs that make user-written import scripts possible later.
- Source aliases and metadata for old-system references when users bring data
  forward.
- Honest code provenance for unmanaged Python.
- Passive setup, environment, and procedure summaries.
- Optional sample/session context that can be corrected after a run.

## Later Scope

- Read-only LAN viewing.
- Measurement-code source setup and approved code update flows.
- Rich sample maps, saved views, comparison, and context correction UX.
- Parameter profiles, calibration records, managed execution, device
  communication, and AI-assisted reviewed automation.

## Key Stories

- US-011: Record code provenance honestly.
- US-012: Register a sample and sample session when useful.
- US-014: Record passive setup context.
- US-016: Record passive procedure context.
- US-017: Migrate a Data Vault-style script to Fricon writers.
- US-018: Start new work in Fricon while old history stays in the old system.
