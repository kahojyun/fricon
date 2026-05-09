# EPIC-005: Migration Ergonomics And Future Lab Scaling

## Status

Draft pending product-analysis revalidation.

## Product Goal

Help labs adopt Fricon for new measurements while old loggers, copied folders,
and historical data remain where they are. This epic protects the initial
adoption slice from becoming a legacy import project or a full automation
stack.

## Initial Adoption Scope

- Data Vault-style script migration guidance for new measurements.
- Source aliases and old-system references as metadata, not primary identity.
- Honest code provenance for interactive unmanaged Python.
- Light contextual summaries for setup, environment, procedure, and parameters.
- Run-bound local configuration copies, snapshots, references, or summaries for
  files and references that old scripts currently leave in copied folders.
- Plain attachment lists for first adoption, without requiring an attachment
  role taxonomy.
- Optional sample/session context that can be corrected after a run.
- No built-in LabRAD dependency, compatibility layer, or unit adapter.
- No claim that Fricon can judge opaque setup, wiring, or instrument context as
  fresh, stale, trusted, or ambiguous without explicit evidence.

## Strategic Follow-On Scope

Detailed strategic follow-on priority and acceptance details live in
`product/future-concepts.md` and
`product/future-stories-and-requirements.md`.

- User-written import helpers for old data when generic APIs mature.
- Read-only LAN viewing.
- Measurement-code source setup and approved code update flows.
- Approved code releases, setup profiles, environment lock files,
  scan-schema helpers, measurement templates, plot presets, export recipes,
  and maintainer handoff for local code changes.
- Managed code/source provenance for measurement, analysis, and calibration.
- Rich sample maps, saved views, comparison, and context correction UX.
- Parameter profiles, effective snapshots, diffs, proposals, and reviewed
  durable promotion of calibration-derived settings.
- Calibration records, chains, working refs, health gates, and proposals.
- Managed execution, device communication, and AI-assisted reviewed automation
  after the code/parameter evidence model is durable.

## Not Initial Adoption

- Whole-notebook capture as the normal provenance mechanism.
- Parsing arbitrary parameter, registry, wiring, or instrument files into a
  Fricon-owned truth model.
- Attachment role taxonomy unless later product evidence shows it is needed.

## Key Stories

- US-011: Record code provenance honestly.
- US-012: Register a sample and sample session when useful.
- US-014: Record passive setup context.
- US-016: Record passive procedure context.
- US-017: Migrate a Data Vault-style script to Fricon writers.
- US-018: Start new work in Fricon while old history stays in the old system.
- US-019: Capture run-bound local configuration.
