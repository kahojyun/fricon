# Future Concepts Ledger

## Status

Supporting future concept ledger for the v0.2 reset.

This is not current behavior, not a task plan, and not an implementation
commitment. Use this file to preserve useful product and architecture context
that is too early for focused proposals, ADRs, or GitHub issues.

Read `README.md` and `design.md` first. This file is only a holding area for
ideas that remain less narrowed than the canonical v0.2 design.

## Purpose

Fricon has several future product areas that depend on the current dataset-first
implementation baseline and the proposed v0.2 data-library reset,
but are not ready for implementation design. This ledger keeps their boundaries
visible so later dataset, measurement, parameter, data-library, and UI work can
avoid decisions that would make those areas harder.

Use this file for short notes only. When a concept becomes active product work,
promote it into a focused design proposal, ADR, or issue plan.

## Promotion Flow

Use this sequence when moving from idea to implementation:

```text
future concept note
  -> focused design proposal
  -> ADR if durable and cross-cutting
  -> issue plan
  -> implementation issues and pull requests
```

Do not create implementation issues directly from this file unless the concept
has been narrowed enough to fit one focused pull request.

## Entry Template

```markdown
## Concept Name

Status: future concept | ADR needed later

Why it matters:

- ...

Boundary:

- Owns ...
- Does not own ...

Likely interfaces:

- ...

Dependencies:

- ...

Open questions:

- ...
```

## Measurement Code Source And Lab Computer Setup

Status: future concept, ADR needed later.

Why it matters:

- Many labs run several acquisition computers for different setups or groups.
  Measurement code often gets copied between those computers, which makes
  maintenance, provenance, and review difficult.
- Fricon should help set up new computers, update shared measurement code, and
  run managed measurements from immutable code snapshots without turning
  ordinary experimenters into Git operators.
- The solution should improve code reuse without requiring a central Fricon
  server or a shared active data library.
- A self-hosted Git service such as Gitea can be the lab's source of truth
  without making Fricon itself the Git host.

Boundary:

- Owns the product model for a configured lab code source or package, approved
  releases/tags, local checkout status, local bare mirrors/caches, immutable
  code snapshots, execution worktrees, environment checks, setup profiles, and
  code provenance recorded on measurements.
- Does not own raw data storage, source-code hosting, branch review workflow,
  secrets management beyond local references, or a full Git client.
- Does not make network storage the active shared database-backed data library
  or the primary editable measurement-code folder.
- Does not claim verified code history for non-managed user-run Python unless a
  managed snapshot or explicit user-supplied summary exists.

Likely interfaces:

- records a code provenance level on measurements, such as unmanaged,
  user-supplied summary, or managed snapshot
- records managed code snapshots with code label, entry point, source kind,
  source label, revision/tag/commit/tree, dirty/hash state, submodule state,
  and environment hints
- exposes setup diagnostics in Fricon Desktop and CLI
- may wrap Gitea, GitLab, GitHub, a bare Git repository, a read-only network
  mirror, package cache, or lab release bundle
- may install or update a local checkout and local Python environment through
  the lab's chosen tools
- may maintain a local bare mirror or cache and expand immutable snapshots into
  temporary execution worktrees for managed ScriptRuns
- links setup profiles, measurement templates, scan-schema helpers, plot
  presets, export recipes, driver/helper modules, and future calibration
  workflow definitions to lab computers
- exports measurement bundles with code provenance and environment summaries
  when available and safe to include

Dependencies:

- v0.2 measurement records and code provenance summaries
- installation/update policy
- Python environment compatibility and diagnostics
- actor/audit boundary for mutating setup actions
- clear local/remote/service ownership model
- future ScriptRun, managed measurement, and runner boundaries

Open questions:

- Should the public concept be named Measurement Code Source, Lab Code Package,
  or Setup Package?
- Which assets should be versioned together, and which should stay local to a
  machine or setup?
- Should Fricon update code directly, call external Git/package tools, or only
  diagnose and link to instructions?
- Should Fricon create the local bare mirror itself, use a user-maintained
  mirror, or support both?
- What exact facts make a managed code snapshot reproducible enough for
  calibration and rerun workflows?
- Should non-managed measurements record only `unmanaged`, or allow
  user-supplied labels and script paths with clear warnings?
- How should dirty local changes be preserved or handed to a maintainer without
  forcing ordinary users through branch and pull-request workflows?
- What is the minimum offline installer or network-mirror story for locked-down
  Windows lab PCs?

## Measurement Record And Future Runner Model

Status: partially promoted to v0.2 for minimal measurement records; future
concept and ADR needed later for managed runner, task queue, resource leases,
script-run provenance, retry/resume, and managed code snapshots.

Focused proposal:

- `experiment-run-and-runner-design.md`

Why it matters:

- Fricon needs explicit measurement records in v0.2 before parameter snapshots,
  code provenance, generated datasets, device state, and notes can be connected
  coherently.
- Measurement records are the natural bridge between Python scripts and later
  desktop inspection, workflow automation, provenance, and AI assistance.
- Runs should preserve reproducibility facts without making the desktop UI the
  primary measurement execution engine too early.
- Code and environment tracking should be explicit run reproducibility context,
  not hidden side effects of dataset writes.

Boundary:

- v0.2 owns measurement identity, lifecycle state, produced dataset links,
  notes/events, optional sample/session context, optional parameter snapshot,
  and honest code provenance level.
- Future runner work owns source script or workflow entry point,
  run-local parameters, parameter bindings, runtime overrides, dataset links,
  effective configuration references, code and environment references, notes,
  lifecycle flags, correction events, task queue records, script-run records,
  and managed code snapshots.
- Does not own parameter history, dataset payload semantics, device driver
  implementation, analysis algorithms, workflow scheduling, or hosted
  repository/account behavior.

Likely interfaces:

- consumes parameter bindings from the parameter registry
- creates or links datasets produced during the run
- records code and environment references when those systems exist
- may consume `uv`, `pixi`, Git, lock-file, checksum, or filesystem metadata
- records instrument snapshot references when device management exists
- exposes run metadata to the desktop UI and Python API
- feeds provenance, reproducibility snippets, and audit summaries

Dependencies:

- durable dataset semantics
- minimal run storage model
- parameter snapshot binding design before parameter-aware runs
- provenance and correction-event policy before immutable run facts harden
- clear policy for Git-backed storage before any managed code-source mirror or
  code snapshot system exists
- environment capture policy that does not make setup opaque

Open questions:

- Which run facts are immutable, and which may receive correction events?
- Should a run record persist a full effective configuration, a hash, or both?
- How should failed, aborted, partially completed, and analysis-only runs be
  represented?
- How should imported or simulation datasets connect to run records?
- How should Fricon capture code history without surprising users or becoming
  a general Git client?
- Which environment facts are useful enough to record by default?
- Should Fricon prefer `uv`, `pixi`, lock-file detection, or passive metadata
  capture first?
- How should missing, dirty, or external code state be represented?

## Sample Or Specimen Identity

Status: partially promoted to v0.2 for lightweight Sample and Sample Session;
future concept for rich sample fields, sample maps, inventory-like behavior,
and cross-session drift workflows.

Why it matters:

- Some experiments are organized around a measured object, batch, preparation,
  condition, or source rather than only a device or parameter set.
- Sample identity should not be hidden inside dataset names, parameter refs, or
  tags.
- Even lightweight sample records can improve filtering, provenance, and
  cross-run comparison without becoming a lab inventory system.

Boundary:

- Owns sample or specimen identity, display labels, lightweight descriptive
  metadata, lifecycle state, and links to runs or datasets.
- Does not own inventory management, custody, permissions, purchasing, storage
  locations, or full LIMS behavior.

Likely interfaces:

- run records may reference a sample or specimen, either from explicit API
  input, active sample/session context, or an attach-later correction
- datasets may inherit sample context from a run
- parameter snapshots or profiles may carry sample context without owning the
  sample record
- desktop filters and compare views may use sample identity

Dependencies:

- minimal run model
- provenance links between runs and datasets
- product decision on how much sample metadata is useful without overbuilding

Open questions:

- What minimal sample fields should be first-class?
- Should sample identity be data-library-local only?
- How should imported datasets with external sample identifiers be handled?
- When is a sample concept unnecessary and a tag or note sufficient?

## Dataset Provenance, Lineage, And Quality State

Status: future concept, ADR needed later.

Why it matters:

- Fricon will need to distinguish measured, imported, processed, simulation,
  and derived datasets when users compare results or trace conclusions.
- Import and export operations should be explainable through source paths, file
  hashes, conversion options, destination summaries, and timestamps.
- Measurement-centered export is a common analysis workflow: a researcher may
  move one measurement bundle to another computer and open it directly without
  creating or importing into another data library.
- Dataset lineage should build on explicit dataset semantics and run records,
  not chart heuristics or file layout details.
- Users need to mark datasets, runs, or outputs as good, suspect, failed,
  calibration, test, invalidated, or superseded without rewriting measurement
  facts.
- Later automation and AI suggestions need explicit quality and invalidation
  context.

Boundary:

- Owns lineage links between datasets, runs, imports, exports, processing
  steps, simulations, source metadata, quality flags, invalidation or
  supersession records, correction links, review notes, and status summaries.
- Owns producer/consumer provenance edges that connect concrete activity
  records, such as measurement, analysis, import, simulation, and calibration
  runs, to their input and output datasets or artifacts.
- Does not own raw dataset payload layout, parameter registry history, or
  analysis algorithms.
- Does not own Arrow payload facts, chart projection semantics, or parameter
  proposal validation.

Likely interfaces:

- dataset records expose kind and lineage references
- import/export workflows record source data library identity, destination,
  conversion, original IDs, export UUIDs, format versions, and checksum
  summaries
- portable measurement bundles can be opened through read-only Python APIs and a
  dedicated GUI viewer without import
- processed datasets link to input datasets and processing context
- analysis, import, simulation, and calibration activity records use shared
  input/output provenance edges instead of forcing all datasets to be owned by
  experiment runs
- dataset and measurement views expose lineage, lifecycle/status badges,
  filters, and provenance summaries
- corrections and invalidations are recorded as events instead of silent edits
- workflow and automation systems can consume lifecycle and validation state
  before promoting results

Dependencies:

- durable dataset semantics
- import/export compatibility policy
- measurement record model for measured datasets
- analysis or processing records for derived datasets
- data-library event or correction model
- UI conventions for flags, filters, and summaries

Open questions:

- Which dataset kinds should be first-class?
- How much import/export provenance is needed for useful reproducibility?
- How should lineage survive dataset archive export and import?
- What is the minimum portable measurement bundle that is useful for offline
  analysis?
- Which lifecycle or validation states apply to datasets, measurements, or
  both?
- Should invalidation block downstream use or only warn?
- How should superseded datasets relate to processed replacements?
- Which status changes require a reason, source, or audit event?

## Analysis Runs And Derived Results

Status: future concept, ADR needed later.

Why it matters:

- Analysis often consumes measured datasets and produces processed datasets,
  figures, metrics, reports, or parameter update proposals.
- Derived results should be connected to the measurement that produced their
  inputs without being stored as children inside the experiment run.
- Automatic calibration needs a clean analysis record before it can explain why
  a parameter proposal was created.

Boundary:

- Owns analysis identity, input links, output links, analysis parameters,
  result summaries, lifecycle or validation state, and links to generated
  datasets, artifacts, reports, or parameter proposals.
- Does not own measurement execution, raw dataset semantics, parameter registry
  commits, workflow scheduling, or device application.

Likely interfaces:

- consumes datasets, experiment runs, parameter snapshots, or artifacts through
  shared input provenance edges
- produces processed datasets, analysis results, figures, reports, metrics, or
  parameter proposals through output provenance edges
- records enough source context for reproducibility without becoming a generic
  notebook-state capture system
- feeds calibration, optimization, benchmark, report, and AI-assistance
  workflows

Dependencies:

- durable dataset semantics
- experiment run model
- shared run input/output provenance edges
- parameter proposal flow for analysis-driven updates
- event or audit log model for accepted corrections or promoted results

Open questions:

- Which analysis outputs should be first-class versus generic artifacts?
- Should simple ad hoc analysis from notebooks create `AnalysisRun` records, or
  should users opt in explicitly?
- How much code and environment summary is useful for analysis without turning
  Fricon into a full notebook or Git history manager?
- How should the desktop UI present downstream analyses from an experiment run?
- Which analysis results may drive automatic calibration proposals?

## Data Library Event And Audit Timeline

Status: future concept, ADR needed later.

Why it matters:

- Corrections, parameter proposals, automation actions, AI-assisted changes,
  imports, exports, run state transitions, and destructive operations all need
  durable audit context.
- A data-library timeline can help users understand what changed without
  exposing storage internals.
- Event history is the preferred direction for correcting completed run facts
  instead of silent mutation.

Boundary:

- Owns append-only or event-like records for user-visible data-library
  mutations, corrections, automation decisions, manual overrides, failed
  automation, and AI-assisted changes.
- Does not own feature-specific business rules, dataset payload storage, or UI
  notification mechanics.

Likely interfaces:

- features emit domain events or audit records for durable actions
- run, parameter, import/export, workflow, AI, and device systems link to event
  IDs where useful
- desktop UI can show filtered event timelines and audit summaries
- export or report flows may include selected audit summaries

Dependencies:

- policy for immutable facts versus correction events
- storage and migration decision for event records
- approval boundaries for automation and AI actions
- feature-specific event vocabularies

Open questions:

- Which actions require durable audit records?
- Should the event log be user-facing from the start or only available through
  summaries?
- How should event records handle privacy-sensitive prompt, path, or note
  content?
- What retention and export behavior is appropriate for local-first data
  libraries?

## Workflow And Calibration Automation

Status: future concept, ADR needed later.

Why it matters:

- Repeated calibration, optimization, benchmark, and scheduled measurement work
  should be explicit instead of hidden inside ad hoc scripts or dataset names.
- Workflow definitions are above individual runs and should preserve versions,
  triggers, expected inputs, generated outputs, approval checkpoints, and
  failure records.
- Calibration is an important workflow family that should connect old
  parameters, managed experiment runs, measured datasets, analysis runs,
  analysis results, proposed parameter changes, validation, and promoted
  snapshots.
- Calibration should not silently mutate `main` or any recommended parameter
  profile during data collection.

Clean automatic calibration should look like:

```text
CalibrationWorkflowDefinition
  -> CalibrationWorkflowRun
      -> ManagedMeasurement produces measured Dataset
      -> AnalysisRun consumes measured Dataset
      -> AnalysisRun produces AnalysisResult and ParameterProposal
      -> ValidationResult checks proposal
      -> Approval or policy gate promotes snapshot to ParameterRef
```

The calibration workflow coordinates the chain. It should not make measured
datasets children of the calibration record, and it should not embed analysis
results inside the original experiment run.

Boundary:

- Owns workflow definitions, workflow versions, workflow runs, triggers,
  schedule metadata, approval checkpoints, manual overrides, and automation
  failure records.
- Owns calibration-specific task shape, analysis-to-parameter proposal rules,
  acceptance criteria, validation summaries, promotion recommendations, and
  review checkpoints.
- Does not own dataset storage, parameter registry commits, device drivers,
  code history, low-level run execution details, raw analysis algorithms, or
  hardware application.

Likely interfaces:

- creates managed measurement runs or run plans
- consumes parameter profiles or bindings
- creates or receives analysis runs, analysis results, and parameter proposals
- creates parameter proposals against a base parameter snapshot
- requests promotion of a validated parameter snapshot to a profile/ref
- may request human approval before mutating parameters, workflows, or devices
- records audit events for automated decisions
- records validation and review metadata

Dependencies:

- experiment run model
- analysis run and derived-result model
- shared run input/output provenance edges
- parameter proposal flow
- analysis result records
- event or audit log model
- local automation approval boundary

Open questions:

- What should a workflow definition contain beyond a Python entry point,
  parameters, schedules, approval checkpoints, and expected outputs?
- Which calibration, optimization, and benchmark workflows should be
  first-class?
- Which automated actions may proceed without human approval?
- How should workflow failures and partial outputs be presented in the desktop
  UI?
- Which calibration results may auto-commit to a calibration profile?
- Which promotions require explicit user review?
- How should stale, low-quality, or out-of-range calibration proposals be
  displayed?
- How should calibration history be plotted or summarized across time?

## Device Apply And Instrument State

Status: future concept, ADR needed later.

Why it matters:

- Parameter state is not the same as hardware state. A device may reject a
  value, round a value, hold stale state, or report readbacks that differ from
  requested settings.
- Future hardware application needs safety checks and auditability without
  turning the parameter registry into a driver framework.

Boundary:

- Owns device identity, declared capabilities, adapter binding, connection
  metadata, command planning, dry runs, application ordering, safety checks,
  readback verification, partial failure handling, and instrument snapshots.
- Does not own parameter snapshot history, parameter diff UI, dataset payload
  storage, or analysis algorithms.

Likely interfaces:

- consumes parameter snapshots or effective run configs
- produces device apply plans and instrument snapshots
- exposes a minimal adapter boundary that can wrap LabRAD, direct Python
  drivers, VISA, serial, vendor SDKs, or dummy devices
- returns readback and failure information to run provenance
- may expose device configuration snapshots for future provenance views

Dependencies:

- experiment run model
- parameter binding and effective config boundary
- event or audit log support
- concrete device workflows before broad abstraction

Open questions:

- What minimal device identity is useful before building driver integration?
- What safety checks are required before applying parameters?
- How should partial hardware apply failures be represented?
- Which device state belongs in run provenance versus long-lived device
  configuration?

## AI-Assisted Automation

Status: future concept, ADR needed later.

Why it matters:

- AI may help with repetitive scientific work such as metadata cleanup,
  parameter comparison, anomaly explanation, workflow drafts, snippet
  generation, and report summaries.
- Mutating actions need explicit review and durable audit records so AI remains
  assistive automation rather than silent authority.

Boundary:

- Owns AI task records, suggestions, review state, accepted/rejected changes,
  model/provider/version metadata when practical, and concise audit summaries.
- Does not own final authority over data, parameters, code, workflow
  definitions, device state, or execution plans.

Likely interfaces:

- reads datasets, runs, parameters, notes, and provenance summaries
- produces suggestions, drafts, reports, patches, or workflow proposals
- routes mutating actions through existing review, proposal, draft, or approval
  boundaries
- records accepted and rejected AI-assisted changes

Dependencies:

- stable run, parameter, dataset, and provenance summaries
- approval and audit model for mutating actions
- privacy and prompt-summary policy

Open questions:

- Which AI actions are suggestion-only?
- Which AI actions may mutate data-library state after explicit approval?
- What model/provider/version metadata should be recorded for
  reproducibility?
- How much prompt or task context can be summarized without storing sensitive
  details unnecessarily?
