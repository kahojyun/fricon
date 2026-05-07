# Legacy Measurement Sample Lessons

## Status

Draft research synthesis.

## Review Date

2026-05-07.

## Sources

Two local legacy measurement sample work directories supplied by the user were
reviewed. Exact paths, local project names, and lab-specific identifiers are
intentionally omitted from this note.

These are concrete lab snapshots, not official framework references. Treat them
as product pressure from real migration-shaped code.

## Observations

The samples show a working but fragile legacy LabRAD-era pattern:

- Measurement identity is spread across Data Vault paths, numeric IDs,
  notebooks, sidecar files, and copied parameter files.
- Hardware/runtime setup is machine-local: LabRAD services, vendor or lab-local
  driver paths, Windows drive paths, static IPs, local registries, and operator
  memory.
- Code versioning is mostly folder copying: backups, old notebooks, nested
  package snapshots, dated JSON, generated caches, and partially used Git.
- Parameters and setup are mutable files such as `parameters.json`,
  `registry.json`, wiring spreadsheets, line/chip info, spectrum CSVs, and
  demod/readout settings.
- Plotting and analysis reconstruct scan meaning after the fact from column
  order, filename conventions, sidecars, and notebook-local arrays.
- Calibration code already contains ad hoc attempts to compare, fit, update, or
  regenerate settings. The problem is that those outputs are mixed with copied
  code folders, mutable parameter files, generated sidecars, and operator
  judgment rather than becoming a reviewed calibration-to-parameter workflow.
- Partial or interrupted acquisition is treated as a cleanup/debugging problem,
  not as a first-class readable lifecycle state.

## Lessons For Fricon

The replacement target is not only LabRAD Data Vault writes. The target is the
informal folder discipline around new measurements.

Fricon should make the following facts first-class for new work:

- `Measurement` identity independent of old paths and numbered titles.
- One or more `DatasetArtifact`s per measurement.
- Explicit scan and trace schema at write time.
- Lifecycle state that makes partial/interrupted data readable.
- Stable Fricon IDs and Python reopen snippets.
- Legacy paths, folders, titles, and numeric IDs as aliases, not identity.
- Honest unmanaged code provenance.
- Passive setup and procedure summaries.
- Run-bound local configuration snapshots or summaries for selected parameter,
  registry, wiring, line/chip, demod/readout, and runner configuration.
- Measurement-centered export that carries semantic context, not just bytes.

The MVP should still avoid LabRAD emulation, old-history import, broad device
control, a full parameter registry, and automatic tracing of every file an
unmanaged script reads.

## Post-MVP Product Pressure

The samples also point beyond the MVP replacement loop:

- Legacy folders are not just storage debt; they are missing experiment memory.
- Repetition currently depends on copied code, mutable config, and operator
  recall.
- The most valuable post-MVP improvement is to make code provenance,
  effective parameter snapshots, generated sidecars, calibration evidence, and
  accepted parameter changes inspectable together.
- Fricon's post-MVP advantage should be reviewed reuse of recorded facts, not
  emulation of legacy paths.
- Read-only compare, handoff, run-like-previous drafts, and failure
  investigation should arrive before mutation-capable automation.
- Parameter proposals should promote effective or fitted settings only after
  review, with before/after diffs and rollback targets where practical.
- Analysis and calibration records should provide evidence for trust decisions
  and proposed parameter changes before they become automation inputs.
- Calibration automation should use chain-scoped working refs or staged
  proposals instead of directly rewriting durable named parameter refs or
  generated config files.
- Automation should grow from trustworthy manifests, snapshots, diffs, and
  review records, not from a generic workflow engine.
- Broad confidence-label taxonomies can wait. The urgent need is durable
  source facts, calibration task health gates, visible diffs, reviewed
  promotion, and an audit trail that lets an experimenter trust the next run.
