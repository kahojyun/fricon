# Future Stories And Requirements

## Status

Proposed v0.3+ planning note.

## Purpose

Capture candidate v0.3+ user stories and product requirements before they
become accepted product scope.

These items are intentionally outside the accepted v0.2 slice. They exist so
v0.2 storage, domain, and API choices do not block likely future needs.

## Planning Stance

v0.3+ should prioritize:

- parameter system first
- managed code source and managed run design early
- run history as a generated view over captured facts
- setup, device, and calibration state as store-and-diff before device apply
- hybrid parameter tree before strict registry
- SDK runner before scheduler

The product rule is: the system captures context automatically where practical;
the user confirms, annotates, or corrects it.

## Candidate v0.3 Epics

FEPIC-001: Parameter System And Snapshot Binding.

- Users manage large parameter sets as named profiles or refs.
- Measurement start resolves refs into immutable effective snapshots.
- Diffs explain what changed between runs, profiles, proposals, and overrides.
- The base model is a structured parameter tree; selected parameters can gain
  typed definitions, units, validation, and richer UI later.

FEPIC-002: Managed Code Source And Run Capture.

- Users configure measurement code sources without copying folders by hand.
- Fricon can capture code snapshot, SDK runner entry point, environment
  summary, stdout/stderr, status, and produced artifacts.
- Ordinary unmanaged Python remains possible, but shows lower provenance
  confidence.

FEPIC-003: Measurement History, Compare, And Handoff.

- Measurement history is assembled from parameter snapshots, code snapshots,
  setup summaries, lifecycle events, operator labels, and artifact links.
- Users compare runs and see what changed since the last session.

FEPIC-004: Setup And Calibration State Store/Diff.

- Users record setup/device/calibration state as structured snapshots and
  ledgers.
- Fricon can diff and bind state to runs without applying settings to devices.

## Candidate User Stories

FUS-001: Maintain named parameter profiles.

- As an experimentalist, I want refs such as `main`, `latest-good`, or
  `cooldown-2026-05` so that large parameter sets have memorable identities.
- Success: refs can move, but each run records the immutable snapshot revision
  actually used.
- Success: ordinary parameters can remain flexible tree nodes; important
  parameters can be promoted to typed definitions with unit and validation.

FUS-002: Bind effective parameters at run start.

- As an experimentalist, I want a measurement to capture the resolved parameter
  snapshot and runtime overrides automatically so that later analysis can
  explain the exact effective settings.
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
- Success: this is a lightweight proposal flow, not a permissions or compliance
  system.

FUS-005: Configure a measurement code source.

- As an experimentalist, I want to register a local or remote code source,
  entry point, and environment hint so that Fricon can stop relying on copied
  folders as the only code provenance story.
- Success: source location, selected revision, environment file, and local
  checkout status are visible before a managed run.

FUS-006: Capture a managed code snapshot.

- As an experimentalist, I want Fricon to capture Git commit, dirty state,
  source hashes, dependency summary, SDK runner entry point, and runner
  invocation when it starts a managed run so that measurement history is not
  hand-entered.
- Success: provenance confidence is visible as unmanaged, observed, or managed
  snapshot.

FUS-007: Run through an opt-in SDK runner.

- As an experimentalist, I want selected scripts to integrate with a Fricon SDK
  runner so that Fricon can capture parameters, code snapshot, stdout, stderr,
  lifecycle, warnings, abort/fail reason, and produced artifacts.
- Success: the unmanaged Python path still works; managed runner is opt-in
  until its safety model is accepted.
- Success: v0.3 runner does not need queues, resource scheduling, retries, or
  broad workflow DAG execution.

FUS-008: Run like a previous measurement.

- As an experimentalist, I want to start from a previous measurement's scan
  schema, parameter snapshot, code source, sample/session context, and plot
  layout so that repeat work is faster without hiding what changed.
- Success: reused facts are copied by reference or snapshot, and changes are
  shown before the new run starts.

FUS-009: Compare two measurements.

- As an experimentalist, I want to compare runs by parameters, code, sample,
  setup, calibration status, lifecycle, notes, and output artifacts so that I
  can explain why results differ.
- Success: comparison is generated from recorded facts, not a manual report.

FUS-010: See operator handoff.

- As a shared-lab user, I want to see what changed since my last session so
  that I can trust the lab computer state before starting work.
- Success: handoff includes parameter ref changes, setup snapshot changes,
  calibration due/expired state, failed runs, imports, exports, and notes.

FUS-011: Store and diff setup snapshots.

- As an experimentalist, I want structured setup snapshots for devices,
  software, firmware, driver versions, connection labels, and readback
  freshness so that setup drift is inspectable.
- Success: Fricon can diff snapshots and bind a snapshot to a run without
  controlling devices.

FUS-012: Track calibration state.

- As an experimentalist, I want calibration records with due dates,
  certificates, as-found/as-left values, pass/fail status, and affected-run
  links so that questionable data can be reviewed.
- Success: out-of-tolerance records can flag a review window without silently
  invalidating data.

## Candidate Product Requirements

FREQ-001: The parameter model is a hybrid structured tree. Flexible nodes are
valid by default, and selected nodes can carry typed definitions, units,
validation, display hints, and documentation.

FREQ-002: Named parameter refs resolve to immutable snapshots before a managed
or unmanaged measurement records data.

FREQ-003: Runtime parameter overrides are recorded separately from profile
snapshots, with actor, time, source, and optional reason.

FREQ-004: Parameter diffs are path-aware, unit-aware where units exist, and
can compare profile, proposal, effective run snapshot, and override layers.

FREQ-005: Parameter profile updates use lightweight proposals with source run
or source snapshot, changed fields, actor, reason, approval/rejection outcome,
and timestamp. They do not require a permissions system in v0.3.

FREQ-006: Managed code provenance records source URI/path, selected revision,
dirty state, source hashes where practical, SDK runner entry point, runner
invocation, and environment summary.

FREQ-007: Fricon shows provenance confidence instead of claiming
reproducibility from Git metadata alone.

FREQ-008: v0.3 managed runner minimum is SDK runner integration. It captures
stdout/stderr excerpts or logs, start/end timestamps, status, abort/fail
reason, produced artifacts, and diagnostic warnings.

FREQ-009: v0.3 managed runner is not a scheduler. Queues, resource management,
retries, and workflow DAG execution are later or ADR-gated.

FREQ-010: Run history and compare views are generated from recorded facts:
parameters, code, setup, lifecycle events, notes, operator labels, and
artifacts.

FREQ-011: Setup/device/calibration state supports store, bind, search, and
diff before any device write-back is implemented.

FREQ-012: Device apply, resumable execution, and AI-assisted mutation require
separate ADRs covering safety, readback, partial failure, and audit behavior.

FREQ-013: v0.3+ remains local-first: no cloud account, hosted dashboard, or
central Fricon server is required for core parameter or run capture workflows.

## Explicit Non-Goals Until ADR-Gated

- Fricon as a full ELN, LIMS, or compliance system.
- Fricon as a Git host or broad package manager.
- Cloud-first experiment tracking dashboards.
- Model registry, hyperparameter leaderboard, or ML sweep controller concepts
  as primary product nouns.
- Device write-back before store/diff is useful and safe.
- Workflow DAG engine as the normal way to run ordinary measurements.
