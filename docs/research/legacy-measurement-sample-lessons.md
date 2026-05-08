# Legacy Measurement Sample Lessons

## Status

Draft research synthesis.

## Review Date

2026-05-08.

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
- Hardware bring-up and instrument calibration are part of the same local
  evidence problem: routines may depend on physical addresses, live output
  state, stop/clear commands, offsets, powers, frequencies, timestamps, and
  later conversion into setting files.
- Analysis handoff is not only Python reopen. The samples also produce
  spreadsheet summaries, presentation decks, plot images, derived arrays, and
  JSON outputs that need traceable links back to input measurements and fitted
  values.
- Advanced feedback and error-correction analysis depends on fragile mappings:
  simulation qubits to physical qubits, readout order, shot groups, IQ
  classification centers, detector events, and observable outputs.
- Partial or interrupted acquisition is treated as a cleanup/debugging problem,
  not as a first-class readable lifecycle state.

## Follow-Up Persona Pressures

A later review of the same sample set clarified several user roles behind the
general migration pressure:

- Calibration and parameter stewardship is current daily work, not only future
  automation. Single-qubit calibration, two-qubit gate calibration, readout
  optimization, feedback tuning, crosstalk work, and hardware bring-up all need
  evidence capture before Fricon owns execution.
- Effective configuration selection is a user problem. When dated parameter
  files, registry backups, lock files, wiring sheets, generated line/chip
  summaries, and temporary sidecars coexist, Fricon should help record which
  state was selected and warn when the context is ambiguous or stale.
- Analysis and reporting users need provenance for secondary artifacts. Fit
  results, plots, spreadsheet rows, presentation slides, and derived data files
  should cite the measurement, selected parameters, code/procedure context, and
  any manual judgment they depend on.
- Advanced experiment analysts need mapping and classification provenance.
  Feedback or error-correction workflows should preserve supplied
  sim-to-physical mappings, readout classification inputs, retraining notes,
  shot grouping, detector/observable formatting, and manual inspection flags.
- Measurement stack authors are a distinct current role. They maintain scan
  helpers, pulse rules, runner integrations, plotting utilities, and report
  generators that other users run without understanding every dependency.

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
- Effective-configuration selection notes and ambiguity/staleness warnings when
  several local files or generated sidecars could plausibly explain a run.
- Optional procedure context for hardware bring-up, instrument calibration,
  readout classification, sim-to-physical mapping, and other unmanaged routine
  facts supplied by the user or integration.
- Links from derived artifacts and reports back to the measurements,
  parameters, code/procedure context, and fitted values they used.
- Measurement-centered export that carries semantic context, not just bytes.

The initial adoption slice should still avoid LabRAD emulation, old-history
import, broad device control, a full parameter registry, and automatic tracing
of every file an unmanaged script reads.

## Strategic Follow-On Product Pressure

The samples also point beyond the initial adoption replacement loop:

- Legacy folders are not just storage debt; they are missing experiment memory.
- Repetition currently depends on copied code, mutable config, and operator
  recall.
- The most valuable strategic follow-on improvement is to make code provenance,
  effective parameter snapshots, generated sidecars, calibration evidence, and
  accepted parameter changes inspectable together.
- Fricon's strategic follow-on advantage should be reviewed reuse of recorded
  facts, not emulation of legacy paths.
- Read-only compare, handoff, run-like-previous drafts, and failure
  investigation should arrive before mutation-capable automation.
- Parameter proposals should promote effective or fitted settings only after
  review, with before/after diffs and rollback targets where practical.
- Analysis and calibration records should provide evidence for trust decisions
  and proposed parameter changes before they become automation inputs.
- Report and derived-artifact records should preserve both visual outputs and
  numeric rationale, including anomaly/status labels when supplied by analysis
  tools.
- Calibration automation should use chain-scoped working refs or staged
  proposals instead of directly rewriting durable named parameter refs or
  generated config files.
- Automation should grow from trustworthy manifests, snapshots, diffs, and
  review records, not from a generic workflow engine.
- Broad confidence-label taxonomies can wait. The urgent need is durable
  source facts, calibration task health gates, visible diffs, reviewed
  promotion, and an audit trail that lets an experimenter trust the next run.
