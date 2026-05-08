# Personas

## Status

Accepted.

## Grounding

These personas were refined against a representative local lab workspace,
which shows current practice built around Windows lab folders, Jupyter
notebooks, LabRAD Data Vault, mutable `parameters.json`/`registry.json` files,
wiring spreadsheets, generated sidecars, calibration scripts, hardware
bring-up helpers, backups, and report artifacts.

The personas describe recurring product responsibilities, not fixed job titles.
One person may move between roles during a single day. A workflow should still
name the role whose decision, risk, or user outcome it primarily serves.

## Story Role Routing

Existing user stories may use broad legacy wording such as "experimentalist".
When stories are revised, choose the most specific primary persona from this
file instead of preserving old role wording by default.

When several roles participate, use the accountable role as the story's primary
persona and mention supporting roles in notes or acceptance criteria. Do not
hide a Fricon technical, analyst, calibration, or measurement-stack concern
behind the generic experimentalist role.

Use the personas this way:

- Use P-001 Experimentalist for ordinary measurement operation: running,
  watching, annotating, recovering, reopening, or exporting new measurement
  work.
- Use P-002 Fricon Technical Maintainer for Fricon development, deployment,
  installation, diagnostics, local runtime health, environment setup, migration
  guidance, support bundles, and shared code setup.
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
uncovers a distinct person with needs that are not already covered by the
Fricon technical maintainer, calibration steward, or measurement stack author
roles.

## Role Boundary Principles

- Roles are responsibility views. A person can switch roles, but each product
  workflow should make the active responsibility explicit.
- MVP stories should prefer the role that directly experiences the MVP problem.
  Post-MVP stories should prefer the role that reviews or owns the durable
  state change.
- Legacy compatibility belongs at role boundaries as aliases, context,
  attachments, summaries, or evidence. It should not create a new role unless
  it creates a durable product responsibility.
- If a story asks Fricon to mutate durable lab state, the responsible role is
  never only P-001 Experimentalist. Use the domain owner for the decision:
  P-004 for parameter or calibration proposals, P-005 for managed-routine or
  measurement-code proposal shape, and P-003 for durable analysis or
  interpretation records. P-002 contributes technical guardrails when a
  proposal depends on Fricon runtime, update, library, or environment safety.

## P-001 Experimentalist

Researchers with entry-level Python ability who run measurement scripts or
notebooks on lab computers. They may use routines written by a more advanced
lab member and may only know enough Python to change a scan range, select
qubits, rerun a cell, or add a note.

Primary responsibilities:

- start and run ordinary unmanaged measurements from Python scripts or
  notebooks
- watch live data and inspect recent measurement outputs
- add lightweight notes, markers, sample/session context, and corrections
- recover, reopen, and export their own measurements for later work
- supply enough local context to explain the run without understanding every
  implementation detail

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

Not responsible for:

- maintaining shared measurement-code sources, runner integrations, or lab
  package environments
- choosing whether calibration-derived values become accepted working
  parameters
- diagnosing every stopped local service, stale SDK, locked library, or
  generated sidecar problem
- making Fricon-managed execution or automation review decisions

Common overlaps:

- uses scripts, scan helpers, and report recipes maintained by P-005
- asks P-002 for setup, launch, environment, or support-bundle help
- hands completed or partial runs to P-003 for analysis and reporting
- becomes P-004 only when evaluating calibration evidence or accepting
  parameter changes

## P-002 Fricon Technical Maintainer

The person who develops, deploys, updates, diagnoses, and supports Fricon for a
lab's local computers and data libraries. This role owns Fricon technical
readiness and integration support, not scientific measurement logic,
calibration decisions, or analysis conclusions.

Primary responsibilities:

- keep Fricon installation, local runtime components, data libraries, Python
  environments, and update paths usable on lab computers
- diagnose setup, compatibility, migration, and support problems before users
  read raw logs
- help labs migrate gradually from old LabRAD/Data Vault, folder, and parameter
  file workflows into Fricon
- define safe support and sharing practices for local code sources and
  environment setup
- provide technical guardrails for post-MVP proposals when Fricon runtime,
  update, data-library, or environment safety matters

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
- technical readiness checks for proposals that depend on local runtime,
  update timing, data-library compatibility, or environment state

Constraints:

- may support Windows, offline, locked-down, or slow-to-update lab computers
- may not be the person who wrote the measurement script or understands the
  scientific intent of a run
- may need redacted support bundles because paths, IP addresses, machine names,
  environment details, and setup files can be sensitive
- may need Fricon to fail before mutation when components, SDKs, or libraries
  are incompatible

Not responsible for:

- operating every measurement run
- writing the measurement stack, pulse rules, runner integrations, or analysis
  code as a product responsibility
- deciding scientific interpretation or whether a fitted value is physically
  trustworthy
- approving parameter, calibration, managed-routine, or analysis proposals as
  the domain owner
- turning mutable local parameter files into accepted calibration outcomes
  without P-004 ownership

Common overlaps:

- supports P-001 when setup or update problems block measurement work
- supports P-005 by making shared code sources and environments usable
- can block or flag proposals that are technically unsafe because of update
  policy, runtime state, environment state, or data-library compatibility
- relies on P-004 for calibration-specific acceptance, rejection, and rollback
  decisions

## P-003 Analyst

The person who reopens completed measurements for notebooks, reports, or HPC
analysis. In practice, this role often produces secondary artifacts such as fit
results, plots, `.npy` or `.json` derived data, spreadsheets, and slide decks.

Primary responsibilities:

- reopen completed, interrupted, or exported measurements through stable
  Fricon APIs and IDs
- inspect datasets, scan schema, metadata, lifecycle, and selected provenance
- produce analysis outputs, derived data, plots, reports, and presentation
  artifacts
- preserve links from derived artifacts back to source measurements,
  parameters, code/procedure context, and manual judgment
- validate supplied mapping, readout-classification, shot-group, or
  detector/observable context for advanced workflows
- review durable analysis, interpretation, or report records when they become
  Fricon-managed evidence instead of notebook-local outputs

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

Constraints:

- may work away from the acquisition computer or after the run has finished
- may need export bundles that redact sensitive paths, setup details, code
  summaries, environment data, or sample context
- may need to distinguish raw measurement facts from notebook-local analysis
  state and manually edited outputs
- may need Fricon to preserve enough context even before analysis records are
  first-class post-MVP objects

Not responsible for:

- operating acquisition or keeping live instruments safe
- maintaining lab runtime, SDK, or package environments
- accepting calibration-derived values into the working parameter state
- approving managed routine or measurement-code changes
- writing or deploying the shared measurement stack, even if analysis code
  uses it

Common overlaps:

- consumes measurements produced by P-001
- may produce evidence that P-004 uses for parameter decisions
- may use report/export recipes maintained by P-005
- may ask P-002 for export privacy or support-bundle guidance

## P-004 Calibration and Parameter Steward

A current lab user who tunes readout, pulse, coupler, crosstalk, demodulation,
feedback, or hardware bring-up parameters and decides whether fitted values
should become the next working local configuration. This role may be the
experimentalist on a small team, but the product pressure is distinct from
ordinary data collection.

Primary responsibilities:

- run or supervise calibration work that produces evidence, fitted values, and
  affected parameter paths
- decide which effective configuration or prior-good state a calibration
  depends on
- evaluate calibration health, safety gates, fit quality, retry/pause needs,
  and manual inspection flags
- mark calibration results as exploratory, accepted, rejected, superseded, or
  needing review
- decide whether proposed parameter changes should become working state, and
  preserve rollback evidence where practical
- review parameter, calibration, and calibration-derived mutation proposals
  before they change durable working state

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

Not responsible for:

- keeping Fricon installed, updated, and diagnosable across lab computers
- writing every measurement helper, runner integration, or report generator
  used by calibration routines
- making generic automation architecture decisions outside calibration and
  parameter review boundaries
- treating mutable JSON files or generated sidecars as the long-term source of
  truth once Fricon has parameter snapshots and proposals

Common overlaps:

- may be the same person as P-001 during daily calibration work
- depends on P-005 for calibration routines, scan helpers, and code provenance
  integration
- may use P-003 analysis outputs as calibration evidence
- relies on P-002 for technical readiness checks when calibration proposals
  affect runtime, update, environment, or data-library compatibility

## P-005 Measurement Stack Author

An advanced Python user who writes or maintains reusable measurement scripts,
scan helpers, pulse-generation rules, runner integrations, plotting utilities,
or export/report recipes used by other lab members.

Primary responsibilities:

- write and maintain user-authored Python code that creates, explains, or
  post-processes measurement work
- integrate Fricon recording into existing scripts, notebooks, Data
  Vault-style helpers, runners, and scan utilities
- declare scan axes, dependencies, trace shapes, repeated shots, validity
  masks, runner identifiers, and procedure summaries from code
- provide reusable plot, report, export, or generated-artifact recipes that
  other roles can run
- preserve honest code provenance without claiming Fricon managed execution
- review managed-routine, measurement-code source, SDK integration, and
  generated-artifact recipe proposals for technical shape and provenance

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

Not responsible for:

- routine operation of every measurement that uses their code
- maintaining Fricon installation, runtime health, or lab update policy as a
  product responsibility
- deciding whether calibration-derived values become accepted working
  parameters
- approving scientific analysis conclusions or calibration health decisions
- making notebooks, copied folders, generated circuits, or report decks the
  canonical system of record

Common overlaps:

- enables P-001 by making scripts and helpers easier to run and record
- helps P-003 produce traceable analysis and report artifacts
- helps P-004 produce calibration evidence and parameter-change proposals
- works with P-002 when shared code sources, package environments, or managed
  entry points become lab-maintained assets
