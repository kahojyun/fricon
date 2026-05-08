# Personas

## Status

Accepted.

## Grounding

These personas were refined against a representative local lab workspace,
which shows current practice built around Windows lab folders, Jupyter
notebooks, LabRAD Data Vault, mutable `parameters.json`/`registry.json` files,
wiring spreadsheets, generated sidecars, calibration scripts, hardware
bring-up helpers, backups, and report artifacts.

The personas describe recurring product roles. One person may move between
roles during a single day.

## Story Role Routing

Existing user stories may use broad legacy wording such as "experimentalist".
When stories are revised, choose the most specific primary persona from this
file instead of preserving old role wording by default.

Use the personas this way:

- Use P-001 Experimentalist for ordinary measurement operation: running,
  watching, annotating, recovering, reopening, or exporting new measurement
  work.
- Use P-002 Lab Maintainer for installation, diagnostics, local runtime
  health, environment setup, migration guidance, support bundles, shared code
  setup, and post-MVP review of proposals that mutate durable lab state.
- Use P-003 Analyst for reopen, analysis, reports, derived artifacts,
  downstream handoff, mapping/classification provenance, and interpretation
  context.
- Use P-004 Calibration and Parameter Steward for calibration evidence,
  effective configuration selection, parameter diffs, fitted values,
  accepted/rejected calibration outcomes, and hardware or instrument
  calibration evidence.
- Use P-005 Measurement Stack Author for SDK shape, script migration,
  Data Vault-style helper translation, scan helpers, runner integrations,
  pulse rules, plotting utilities, and report/export recipes.

Do not add a separate future-automation persona unless a future product slice
uncovers a distinct person with needs that are not already covered by the lab
maintainer, calibration steward, or measurement stack author roles.

## P-001 Experimentalist

Researchers with entry-level Python ability who run measurement scripts or
notebooks on lab computers. They may use routines written by a more advanced
lab member and may only know enough Python to change a scan range, select
qubits, rerun a cell, or add a note.

Needs:

- low-boilerplate Python recording
- live feedback while a measurement is running
- the ability to monitor more than one live measurement or view without
  disturbing acquisition
- reliable recovery from interrupted runs
- easy reopening and export for later analysis
- visible active sample/session, measurement name, scan axes, units,
  dependencies, and selected local configuration context
- no requirement to understand Rust, IPC, storage layout, or database internals
- no requirement to understand every copied script, generated sidecar,
  parameter backup, or external runner detail before starting new work

Constraints:

- may use Windows lab computers
- may work on locked-down or offline machines
- may pin Python environments with `uv.lock`, virtual environments, or lab
  setup scripts
- may not update Desktop, local runtime, CLI, and Python SDK together
- may need Fricon to fail safely during long-running measurements, exports, or
  local maintenance work
- may rely on Jupyter, Conda or `uv` environments, LabRAD/Data Vault services,
  and local folders with dated copies or backups during migration

Observed real-case pressures:

- startup can be a manual sequence of local services and Jupyter
- measurement data may be saved by dataset IDs and paths that are meaningful
  locally but fragile for handoff
- interrupted sweeps and long-running saves need readable partial state, not
  only a final success/failure bit

## P-002 Lab Maintainer

The person who helps keep lab measurement code, setup context, and computers
usable.

Needs:

- clear installation and diagnostics
- explicit split between Fricon install, data-library location, Python
  environment, and later measurement-code management
- support for sharing approved measurement code without copying random folders
- exportable support bundles that are local and redacted by default
- setup/update guidance that works for offline, locked-down, or slow-to-update
  lab computers
- user-facing diagnostics before raw logs, especially for stopped runtime
  components, locked libraries, stale SDKs, incompatible versions, and
  migration-required states
- effective-configuration diagnostics when several dated parameter files,
  registry files, lock files, wiring sheets, generated line/chip summaries, or
  local backups may all appear plausible
- a way to explain which local services, data paths, setting paths, package
  environments, and generated sidecars were expected during a run
- migration guidance that lets old LabRAD/Data Vault history remain in place
  while new work records honest Fricon context
- post-MVP review responsibility for parameter, calibration, managed-routine,
  or automation proposals before they mutate durable lab state

## P-003 Analyst

The person who reopens completed measurements for notebooks, reports, or HPC
analysis. In practice, this role often produces secondary artifacts such as fit
results, plots, `.npy` or `.json` derived data, spreadsheets, and slide decks.

Needs:

- stable IDs and Python reopen snippets
- direct read access to export bundles
- simple manifest or index previews in exported bundles
- metadata and units preserved well enough to understand the data away from the
  acquisition computer
- privacy-aware export choices for local paths, code provenance, environment
  summaries, setup details, and sample context
- enough lifecycle, parameter, code, and procedure context to tell whether an
  analysis result came from a completed run, an interrupted run, a calibration
  retry, or a manually edited notebook state
- links from derived artifacts back to the measurement inputs they used, even
  before analysis attempts become first-class post-MVP records
- report handoff that does not depend on reconstructing local folder state or
  remembering which notebook cell produced a plot
- enough mapping, readout-classification, and artifact provenance to trust
  advanced workflows where simulation qubits, physical qubits, shot groups,
  detector events, and fitted metrics must stay aligned

## P-004 Calibration and Parameter Steward

A current lab user who tunes readout, pulse, coupler, crosstalk, demodulation,
feedback, or hardware bring-up parameters and decides whether fitted values
should become the next working local configuration. This role may be the
experimentalist on a small team, but the product pressure is distinct from
ordinary data collection.

Needs:

- clear links between calibration measurements, input parameter snapshots,
  generated sidecars, fitted outputs, and any later parameter-file edits
- safe capture of selected `parameters.json`, `registry.json`, wiring,
  demodulation, readout, line/chip, and runner summaries at run time
- diffs or summaries that explain what changed since a previous good
  calibration or measurement
- a way to mark calibration results as exploratory, accepted for current work,
  rejected, superseded, or needing review
- exported evidence that can explain why a parameter changed without exporting
  sensitive local paths by default
- prerequisite and safety checks before long calibration routines, including
  missing registry values, invalid fitted inputs, alignment failures,
  instrument output state, and known-good backup state
- before/after records for hardware or instrument calibration work, including
  offsets, powers, frequencies, timestamps, and cleanup/stop outcomes when
  supplied by the unmanaged routine

Constraints:

- may work with mutable JSON files, lock files, dated backups, copied setting
  folders, generated temporary files, and local database helpers
- may need to compare or restore a previous parameter state after a failed
  calibration
- may run routines that update local parameters while also producing datasets,
  so mutation and measurement facts must stay distinguishable
- may depend on physical instrument addresses, external device state, and
  operational cleanup that Fricon can record but not control in the MVP
- should not need Fricon to own device control before Fricon can record the
  calibration evidence honestly

## P-005 Measurement Stack Author

An advanced Python user who writes or maintains reusable measurement scripts,
scan helpers, pulse-generation rules, runner integrations, plotting utilities,
or export/report recipes used by other lab members.

Needs:

- SDK patterns that fit ordinary Python modules, scripts, and notebooks without
  requiring Fricon-managed execution
- a way to declare scan axes, dependencies, trace shapes, repeated shots,
  validity masks, runner identifiers, and procedure summaries from code
- honest code provenance for copied folders, local changes, Git state,
  packaged helpers, generated circuits, and imported utility modules
- failure and cleanup semantics that keep instruments, local runtime state, and
  partially recorded datasets understandable after errors or interrupts
- migration ergonomics for translating Data Vault-style helpers into Fricon
  recording without rewriting the whole measurement stack at once

Constraints:

- may be responsible for scripts that other users run without understanding
  every internal dependency
- may need Fricon to record context without claiming reproducibility it cannot
  guarantee for unmanaged execution
- may create post-MVP pressure for approved code sources, managed entry points,
  templates, and reviewed updates, but those are not MVP promises
