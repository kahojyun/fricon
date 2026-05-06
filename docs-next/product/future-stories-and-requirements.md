# Future Stories And Requirements

## Status

Proposed post-MVP planning backlog.

## Purpose

Capture candidate post-MVP user stories and product requirements before they
become accepted product scope.

These items are intentionally outside the accepted MVP. They exist so MVP
storage, domain, and API choices do not block likely future needs.

This file uses priority horizons, not semantic-version labels. A compatible
feature can still ship on the same release line as the MVP if the relevant
compatibility, storage, and API policies allow it.

## Planning Stance

Post-MVP Fricon should move from data-library replacement toward local
experiment memory and reviewed action. The product should capture enough facts
to explain, compare, hand off, replay, and safely automate lab work without
pretending to own code, parameters, setup, or devices before those boundaries
are designed.

Post-MVP work should prioritize:

- parameter system first
- managed run second
- reviewable automation workflow third

Analysis of representative legacy measurement workflows suggests read-only
comparison, run like a previous measurement, and failure investigation should
arrive before mutation-capable automation. These workflows can build trust from
recorded facts without taking control of parameters, devices, or code.

The dominant pain is code and parameter management disorder. Copied measurement
folders, mutable parameter files, generated sidecars, and notebook-local
analysis make it hard to know which settings produced a calibration result.
Post-MVP Fricon should therefore prioritize durable code provenance, effective
parameter snapshots, parameter diffs, analysis evidence, and reviewed
calibration proposals before investing in broad automation.

The product rule is: the system captures context automatically where practical;
the user confirms, annotates, or corrects it. Any workflow that mutates
parameters, setup, devices, code, or data-library state needs explicit preview,
review, and audit semantics.

Prefer features that make experiments explainable, comparable, repeatable, or
safely reviewable. Deprioritize features that merely imitate legacy acquisition
tools, add device control without recorded state, or create automation before
Fricon can explain the facts automation depends on. Detailed field-level
confidence labels are a later refinement unless a concrete workflow requires
them; the first priority is recording the source facts and review decisions
that calibration and repeat work depend on.

## Priority 1: Parameter System

FEPIC-001: Parameter System And Snapshot Binding.

- Users manage large parameter sets as named profiles or refs.
- Measurement start resolves refs into immutable run-bound snapshots.
- Diffs explain what changed between runs, profiles, proposals, and overrides.
- The base model is a hybrid parameter tree: flexible structured values first,
  with selected parameters upgraded to typed definitions, units, validation,
  display hints, and UI affordances.
- Table sections can expose user-defined row keys that later visualization
  tools can treat as target keys.

FUS-001: Maintain named parameter profiles.

- As an experimentalist, I want refs such as `main`, `latest-good`, or
  `cooldown-2026-05` so that large parameter sets have memorable identities.
- Success: refs can move, but each run records the immutable snapshot revision
  actually used.
- Success: ordinary parameters can remain flexible tree nodes; important
  parameters can be promoted to typed definitions with unit and validation.

FUS-002: Bind effective parameters at run start.

- As an experimentalist, I want a measurement to capture the resolved parameter
  snapshot and runtime overrides so that later analysis can explain the exact
  effective settings.
- Success: overrides are separate facts with actor, time, source, and optional
  reason.

FUS-003: Compare parameters across runs and proposals.

- As an experimentalist, I want to diff two runs, snapshots, or profiles so
  that I can find parameter drift without reading JSON files by hand.
- Success: diffs preserve path, value, unit, source profile, override status,
  actor, and linked run or proposal.

FUS-004: Promote a good run into a parameter proposal.

- As an experimentalist, I want to turn the effective settings from a successful
  run into a reviewed proposal so that useful adjustments can become a named
  profile intentionally.
- Success: promotion records source run, changed fields, reviewer/actor, and
  approval or rejection outcome.
- Success: promotion can cite analysis or calibration evidence when the source
  run produced fitted settings.
- Success: proposal review is distinct from run-like-previous convenience; a
  named parameter ref changes only after explicit review.
- Success: this is a lightweight proposal flow, not a permissions or compliance
  system.

FUS-008: Run like a previous measurement.

- As an experimentalist, I want to start from a previous measurement's scan
  schema, parameter snapshot, code source, sample/session context, and plot
  layout so that repeat work is faster without hiding what changed.
- Success: reused facts are copied by reference or snapshot, and changes are
  shown before the new run starts.
- Success: the new run starts from a draft with visible differences from the
  source run; nothing starts or mutates until the user confirms.
- Success: unmanaged source facts remain labeled unmanaged instead of being
  promoted to managed provenance.
- Success: calibration-derived parameters remain tied to their source evidence
  instead of becoming anonymous copied values.

FUS-015: Use parameter row keys as visual targets.

- As an experimentalist, I want parameter table row keys to carry enough
  user-defined target identity that a sample visualization can locate objects
  without Fricon imposing a sample-component ontology.
- Success: a row key can be matched to a user-authored sample-map config, DSL,
  or lab script when such a view exists.
- Success: color maps and numeric labels can be derived from parameter snapshot
  queries, not from hardcoded sample object fields.
- Success: a visualizer definition can be discoverable near the relevant sample
  or session without making its shape model part of core sample identity.
- Success: unmatched or ambiguous keys remain visible as data-quality issues,
  not hidden failures.

Parameter requirements:

- FREQ-001: The parameter model is a hybrid structured tree. Flexible nodes are
  valid by default, and selected nodes can carry typed definitions, units,
  validation, display hints, and documentation.
- FREQ-002: Named parameter refs resolve to immutable snapshots before a
  managed or unmanaged measurement records data.
- FREQ-003: Runtime parameter overrides are recorded separately from profile
  snapshots, with actor, time, source, and optional reason.
- FREQ-004: Parameter diffs are path-aware, unit-aware where units exist, and
  can compare profile, proposal, effective run snapshot, and override layers.
- FREQ-005: Parameter profile updates use lightweight proposals with source run
  or source snapshot, changed fields, actor, reason, approval/rejection
  outcome, and timestamp. They do not require a permissions system in the first
  parameter workflow slice.
- FREQ-023: Calibration-derived parameter changes must flow through parameter
  proposals or calibration proposals. Direct edits to active parameter refs
  should not be the normal automation path.
- FREQ-016: Parameter table rows can expose stable, user-defined target keys
  for visualization and compare views. Fricon must not require those keys to
  imply a first-class physical sample-component model.
- FREQ-020: Sample visualizers, when introduced, should be modeled as views
  over parameter snapshots and snapshot query results. Their storage location,
  query language, and schema-evolution policy remain deferred until a dedicated
  spec or ADR.

Not first parameter slice:

- Device write-back.
- Strict global registry for every parameter.
- Permissions, compliance, or multi-user approval system.
- Automatic tracing of every parameter read without explicit design.
- A mandatory field-level confidence taxonomy for every parameter value.
- Heavyweight sample-component ontology.
- Mandatory 2D sample-map authoring.
- Visualizer schema migration or compatibility policy.

## Priority 2: Managed Run

FEPIC-002: Managed Code Source And Run Capture.

- Users configure measurement code sources without copying folders by hand.
- Users can mark importable Python entry points for opt-in Fricon management.
- Fricon can capture code snapshot, SDK runner entry point, environment
  summary, stdout/stderr, status, diagnostics, and produced artifacts.
- Fricon can assemble a compact run manifest for compare, handoff, export, and
  failure investigation.
- Managed run improves provenance coverage; it does not by itself guarantee
  scientific reproducibility without parameter, setup/device, environment, and
  calibration coverage.
- Ordinary interactive unmanaged Python remains possible, but shows lower
  provenance coverage.

FEPIC-003: Measurement History, Compare, And Handoff.

- Measurement history is assembled from parameter snapshots, code snapshots,
  setup summaries, lifecycle events, operator labels, and artifact links.
- Users compare runs and see what changed since the last session.

FUS-005: Configure a measurement code source.

- As an experimentalist, I want to register a local or remote code source,
  entry point, and environment hint so that Fricon can stop relying on copied
  folders as the only code provenance story.
- Success: source location, selected revision, environment file, and local
  checkout status are visible before a managed run.
- Success: code-source setup can replace copied working folders as the normal
  explanation for where measurement code came from.

FUS-006: Capture a managed code snapshot.

- As an experimentalist, I want Fricon to capture Git commit, dirty state,
  source hashes, dependency summary, SDK runner entry point, and runner
  invocation when it starts a managed run so that measurement history is not
  hand-entered.
- Success: provenance level is visible as unmanaged, observed, or managed
  snapshot.
- Success: calibration evidence records which code snapshot or unmanaged code
  summary produced the analysis or fitted values.

FUS-007: Run through an opt-in SDK runner.

- As an experimentalist, I want selected scripts to integrate with a Fricon SDK
  runner so that Fricon can capture parameters, code snapshot, stdout, stderr,
  lifecycle, warnings, abort/fail reason, and produced artifacts.
- Success: the unmanaged Python path still works; managed runner is opt-in
  until its safety model is accepted.
- Success: the first managed-runner slice does not need queues, resource
  scheduling, retries, or broad workflow DAG execution.

FUS-009: Compare two measurements.

- As an experimentalist, I want to compare runs by parameters, code, sample,
  setup, calibration status, lifecycle, notes, and output artifacts so that I
  can explain why results differ.
- Success: comparison is generated from recorded facts, not a manual report.
- Success: a previous-good run can be selected as a baseline, and missing or
  incomplete facts are shown rather than hidden.

FUS-010: See operator handoff.

- As a shared-lab user, I want to see what changed since my last session so
  that I can trust the lab computer state before starting work.
- Success: handoff includes parameter ref changes, setup snapshot changes,
  calibration due/expired state, failed runs, imports, exports, and notes.
- Success: handoff distinguishes facts, warnings, decisions, and missing
  provenance so the next operator knows what remains uncertain.

FUS-016: Inspect a run manifest.

- As an experimentalist, I want a run manifest that links the available
  measurement, parameter snapshot, row/target keys, code/environment summary,
  lifecycle, logs, artifacts, operator, and timestamps so that I can understand
  a run without opening several unrelated stores.
- Success: the manifest can be opened from Desktop, Python, and export bundles.
- Success: the manifest states provenance coverage instead of implying that
  unmanaged work was fully captured.
- Success: the manifest is a composite view over recorded facts; it does not
  own or duplicate parameter, code, setup, analysis, or artifact records.

FUS-017: Investigate a failed fit or anomalous measurement.

- As an experimentalist, I want to follow a suspicious result back to inputs,
  parameter changes, code/environment state, setup labels, logs, and analysis
  attempts so that tedious failure investigation is not a manual archaeology
  task.
- Success: failed or questionable analysis/fit attempts can be recorded with
  input artifacts, method/code reference, quality metrics, failure reason, and
  produced outputs when available.
- Success: comparison to a previous-good run is generated from recorded facts.
- Success: investigation can show whether code, parameter snapshot, generated
  sidecars, or calibration evidence changed from the previous-good baseline.

FUS-019: Record measurement intent and outcome.

- As an experimentalist, I want to record the question, intent, outcome, and
  trust decision for a measurement so that future compare, handoff, and repeat
  work can use more than raw data and filenames.
- Success: intent can be recorded before or during a run, and outcome can be
  recorded after review.
- Success: outcomes can link to datasets, analysis attempts, notes, and
  lifecycle events.
- Success: lightweight labels such as accepted, questionable, invalidated, or
  repeat-needed can explain review state without becoming a full ELN.

Managed-run requirements:

- FREQ-006: Managed code provenance records source URI/path, selected revision,
  dirty state, source hashes where practical, SDK runner entry point, runner
  invocation, and environment summary.
- FREQ-007: Fricon shows provenance level and coverage instead of claiming
  reproducibility from Git metadata alone.
- FREQ-008: The first managed-runner slice is SDK runner integration. It
  captures stdout/stderr excerpts or logs, start/end timestamps, status,
  abort/fail reason, produced artifacts, and diagnostic warnings.
- FREQ-009: The first managed-runner slice is not a scheduler. Queues, resource
  management, retries, and workflow DAG execution are later or ADR-gated.
- FREQ-010: Run history and compare views are generated from recorded facts:
  parameters, code, setup, lifecycle events, notes, operator labels, and
  artifacts.
- FREQ-017: A run manifest links the durable measurement identity to parameter
  snapshot/ref facts, row/target keys when present, code/environment summary,
  lifecycle/log events, artifacts, operator, timestamps, and provenance
  coverage.
- FREQ-018: Analysis or fit attempts can be represented as investigation
  records with inputs, method/code reference, status, diagnostics, quality
  metrics, failure reason, and outputs when available.
- FREQ-021: Measurement intent and outcome records preserve question, decision,
  linked evidence, actor, timestamp, and review state without turning Fricon
  into a full ELN.
Not first managed-run slice:

- Scheduler, queues, resource leases, broad retry policy, or workflow DAG.
- Resumable scan-point execution.
- Making managed execution mandatory for ordinary exploratory scripts.
- Treating shell-command wrapping as the primary runner UX.

## Priority 3: Reviewable Automation Workflow

FEPIC-004: Setup And Calibration State Store/Diff.

- Users record setup/device/calibration state as structured snapshots and
  ledgers.
- Fricon can diff and bind state to runs without applying settings to devices.
- Setup snapshots, device snapshots, and calibration ledgers are related but
  distinct: setup records declared or passive context; device snapshots record
  identity and observed/readback facts when available; calibration ledgers
  record validity and review state.
- The first calibration-state value is not autonomous device control. It is
  making calibration evidence, affected parameters, and applied changes
  inspectable enough that users can trust the next run.

FEPIC-005: Reviewable Automation Workflow.

- Users can preview automation actions before they mutate Fricon state, device
  state, parameter refs, or calibration records.
- Automation proposals record intended inputs, expected mutations, affected
  objects, actor, review outcome, and audit events.
- Automation grows from parameter system and managed run records instead of
  becoming an unrelated workflow engine.
- Automation records are distinct from managed measurement execution: reusable
  recipes, previewable proposals, review decisions, and execution records each
  own different facts.

FUS-011: Store and diff setup snapshots.

- As an experimentalist, I want structured setup snapshots for devices,
  software, firmware, driver versions, connection labels, and readback
  freshness so that setup drift is inspectable.
- Success: Fricon can diff snapshots and bind a snapshot to a run without
  controlling devices.
- Success: readback freshness states whether it is user-supplied,
  integration-supplied, or device-observed, with source and timestamp.

FUS-012: Track calibration state.

- As an experimentalist, I want calibration records with due dates,
  certificates, as-found/as-left values, pass/fail status, and affected-run
  links so that questionable data can be reviewed.
- Success: out-of-tolerance records can flag a review window without silently
  invalidating data.
- Success: a calibration record can link to the parameter proposal or
  calibration proposal that applied its derived settings.
- Success: fit quality, residuals, warnings, and manual rationale can be
  preserved when they materially affect whether a calibration should be used.

FUS-020: Stage calibration results before applying parameter changes.

- As an experimentalist, I want fitted calibration outputs to become reviewed
  proposals before they update active parameters so that automation does not
  silently overwrite the state future runs depend on.
- Success: a proposal records source measurements, analysis attempts, fitted
  values, generated sidecars or derived artifacts where relevant, affected
  parameter paths, before/after diff, code context, and reviewer/actor.
- Success: acceptance creates or updates the target parameter profile/ref
  through the same proposal/audit path as manual parameter promotion.
- Success: rejection or deferral preserves the evidence without mutating active
  parameter state.

FUS-013: Preview an automation workflow.

- As an experimentalist, I want a proposed automation workflow to show what it
  will read, run, and mutate before execution so that hidden state changes do
  not surprise me.
- Success: preview separates parameter changes, setup/device changes, managed
  runs, analysis steps, calibration proposals, and data-library mutations.
- Success: preview identifies whether each action is read-only, produces a
  durable record, mutates Fricon state, or reaches an ADR-gated device boundary.
- Success: calibration automation previews generated artifacts, affected
  parameter paths, before/after diffs, and rollback targets before applying
  accepted changes.

FUS-014: Review and apply automation proposals.

- As a lab maintainer, I want automation proposals to require explicit review
  before mutating durable lab state so that automation remains accountable.
- Success: approval, rejection, actor, reason, timestamp, and affected objects
  are recorded.

FUS-018: Replay a routine recipe after review.

- As an experimentalist, I want common compare, run, and analyze routines to be
  captured as reviewable recipes so that repetitive work is faster without
  hiding what will change.
- Success: preview shows parameter refs or overrides, code entry point,
  expected artifacts, analysis steps, and durable mutations before execution.
- Success: replay states which routine/code version, parameter snapshot,
  calibration evidence, generated sidecars, and safety envelope it depends on.
- Success: read-only batch compare or triage can be useful before
  mutation-capable automation exists.
- Success: replay records recipe template, automation proposal, review
  decision, execution status, produced records, and failure handling separately.

Automation requirements:

- FREQ-011: Setup/device/calibration state supports store, bind, search, and
  diff before any device write-back is implemented.
- FREQ-012: Device apply, resumable execution, and AI-assisted mutation require
  separate ADRs covering safety, readback, partial failure, and audit behavior.
- FREQ-013: Post-MVP improvements remain local-first: no cloud account, hosted
  dashboard, or central Fricon server is required for core parameter, run, or
  automation workflows.
- FREQ-014: Reviewable automation must produce a preview of intended reads,
  writes, device interactions, generated records, and failure handling before
  mutation starts.
- FREQ-015: Automation records must preserve actor, trigger, reviewed plan,
  accepted or rejected changes, execution status, produced artifacts, and audit
  events.
- FREQ-019: Routine recipes and reviewed replay use parameter snapshots and
  run manifests as inputs. Read-only compare or triage may run first; mutation
  requires preview, review, execution status, and audit records.
- FREQ-022: AI may summarize, propose, or assist, but mutating actions enter as
  ordinary reviewed proposals. Durable AI-created conclusions record model or
  tool provenance and privacy-scoped inputs where practical.
- FREQ-024: Calibration proposals preserve source measurements, analysis
  attempts, fitted values, affected parameter paths, before/after diffs, code
  context, review outcome, and rollback target where practical.
- FREQ-025: Automation previews classify intended effects at least as
  read-only inspection, derived artifact creation, data-library mutation,
  parameter/profile mutation, device/hardware mutation, or ADR-gated
  OS/network mutation.
- FREQ-026: Generated sidecars or derived config files that influence analysis,
  calibration, or replay are recorded as artifacts with source inputs,
  generator/code context, and links to the proposal or run that depends on
  them.

Not first automation slice:

- AI autonomous mutation.
- Unreviewed recipe replay that mutates parameters, setup, devices, code
  sources, or data-library state.
- Device write-back before store/diff, readback, safety, and partial-failure
  behavior are accepted by ADR.
- Fully automatic calibration writeback before code provenance, parameter
  proposals, calibration evidence, and rollback paths are durable.
- Generic workflow DAG engine as the normal way to run ordinary measurements.
- Cloud-first dashboard, account, or team-permission system.

## Supporting Candidates

- Read-only LAN monitoring for local lab viewing without remote writes.
- User-defined 2D sample-map configs/DSLs that map parameter row keys or user
  labels to visual regions, plus saved comparison views.
- External large asset references for detector files, images, and waveforms.
- User-facing dataset streams, if internal stream support proves useful enough
  to expose later.
