# Future Concepts

## Status

Accepted post-MVP priority ledger.

## Purpose

Preserve important post-MVP directions without letting future systems inflate
the MVP measurement loop.

These are product priority horizons, not semantic-version promises. Compatible
capabilities may ship on the same release line as the MVP if the compatibility,
storage, and API policies allow it.

Post-MVP concepts should move Fricon toward local experiment memory and
reviewed action. The point is to explain, compare, hand off, repeat, and safely
automate work from recorded facts, not to imitate legacy acquisition tools or
make device control, sample visualization, or AI the product center by itself.

## Promotion Rule

A future concept can move into implementation only after it has a clear story,
domain owner, ADR if needed, and does not make the core measurement loop harder
to use.

The priority order is a planning default, not a release-number commitment. A
lower-priority item can move earlier only when it is small, compatible, and does
not weaken the higher-priority product model.

## Priority 1: Parameter System

Why first:

- Fricon's core motivation is dissatisfaction with existing parameter
  management and code management around physical measurements.
- The most painful legacy failure mode is not a missing label taxonomy; it is
  copied code and mutable parameter/config files making it unclear which
  settings a calibration depended on or changed.
- Parameter state directly shapes whether a completed measurement can be
  understood, compared, repeated, or promoted into better lab practice.
- Managed runs and automation workflows need a parameter model to avoid
  recording only half of the experiment record.

Included concepts:

- FC-004: Parameter profiles, immutable run-bound snapshot binding, history,
  proposals, diff views, and table row keys usable as lightweight target keys.
- FC-014: Measurement history and compare views generated from captured
  parameter, setup, lifecycle, code, and artifact facts.

Boundary:

- Start with a hybrid parameter tree: flexible structured values first, with
  selected parameters upgraded to typed definitions, units, validation, and UI
  affordances.
- Use lightweight proposals with actor, reason, approval history, before/after
  diffs, source evidence, and rollback target where practical.
- Treat calibration-derived values as proposed parameter changes first. A
  calibration can produce evidence, fitted values, and affected parameter
  paths without directly mutating a durable named profile.
- Treat sample target binding as a convention first: parameter table row keys
  may be reused by visualization code and snapshot queries, but Fricon does
  not need a mandatory sample-component ontology in the first parameter slice.
- Do not require a full permissions system, strict global registry, or device
  write-back for the first parameter slice. A mandatory field-level confidence
  taxonomy is also deferred unless a concrete workflow proves it is needed.

## Priority 2: Managed Run

Why second:

- Managed run turns the SDK from an honest unmanaged logger into a higher
  provenance experiment runner when users opt in.
- Useful run history depends on captured code, parameters, lifecycle, logs, and
  produced artifacts, not manual entry.
- Calibration evidence needs to cite the code source or code snapshot that
  produced fitted values; otherwise automated calibration only formalizes the
  old copied-folder ambiguity.
- Managed run should grow from ordinary importable Python code, not from a
  separate experiment DSL.

Included concepts:

- FC-003: Measurement-code source setup that replaces copied-code folders with
  approved-code update flows, local checkout/environment guidance, and no
  central Fricon server.
- FC-007: Managed code snapshots and opt-in managed run capture.
- FC-014: Operator handoff and run history views generated from captured facts.
- FC-017: Run manifest and investigation record linking parameter snapshot,
  target keys, code/environment summary, lifecycle, logs, artifacts, and
  failure/anomaly evidence.

Boundary:

- The first managed-run slice is SDK runner integration, not a blind
  shell-command wrapper and not a scheduler.
- Ordinary interactive unmanaged Python remains valid and clearly labeled.
- Capture enough context for compare and failure investigation before designing
  queues, resumable scan points, or autonomous automation.
- Managed run should replace copied-code folders as a provenance workflow
  before it tries to own scheduling or hardware orchestration.
- Queues, resource leases, retries, workflow DAGs, and resumable execution are
  later or ADR-gated.

## Priority 3: Calibration Chains And Reviewable Automation

Why third:

- Automation should reuse the parameter system and managed run record instead
  of becoming an independent workflow engine.
- The experiment UX risk is hidden mutation. The product value is preview,
  review, audit, and explicit approval around changes to parameters, setup,
  code, devices, or data-library state.
- Calibration automation is valuable early only when it reduces parameter
  confusion: preview the analysis inputs, generated sidecars or derived
  artifacts, fitted values, affected parameter paths, task health, retry or
  pause decisions, and rollback path before durable changes are promoted.
- A later managed measurement model can borrow from declarative UI and
  infrastructure systems: users describe expected parameter-derived setup or
  device state, Fricon computes a diff from observed state, previews an apply
  plan, and only then performs safe parallel or ordered writes.
- AI-assisted workflow is only acceptable after the audit and review model
  exists.

Included concepts:

- FC-005: Analysis records that consume artifacts and produce derived outputs.
- FC-006: Calibration records with reviewable proposals.
- FC-008: Device identity and managed communication.
- FC-009: Managed measurement plans.
- FC-015: Workflow preview layer for calibration, benchmark, and reviewed
  automation flows.
- FC-010: AI-assisted automation after the audit model exists.
- FC-018: Routine recipes, reviewed replay, and batch compare for repetitive
  work after parameter and run facts are trustworthy.
- FC-019: Desired-state setup/device planning, reconciliation diffs, and
  reviewed apply plans for routines where imperative nested loops are too
  error-prone.

Boundary:

- Store and diff setup, device, and calibration state before applying settings
  back to devices.
- Keep imperative Python scripts valid. Desired-state planning is a higher
  provenance option for routines that can declare pure parameter-to-state
  functions and explicit hardware constraints.
- Calibration workflows for sample/control parameters should first create
  durable evidence, fitted values, health decisions, and promotion proposals.
  A bootstrap calibration sequence should be able to continue through healthy
  substeps by updating a chain-scoped calibration working ref, without asking
  for approval at every small update. Publishing selected results to durable
  named profiles remains a separate promotion step.
- Instrument/device calibration and desired-state device apply are separate
  device/setup capabilities with their own safety model.
- Device write-back, resumable execution, and AI-assisted mutation need safety,
  readback, partial-failure, and audit ADRs.
- Reconciliation must not assume device writes are commutative, idempotent, or
  safe to parallelize. Apply plans need dependency, settling, readback,
  timeout, and abort semantics before they can reach hardware.
- Read-only compare, triage, and preview workflows may arrive before
  mutation-capable automation if they reuse the same record and review model.
- A broad workflow DAG engine is not the normal way to run ordinary
  measurements.

## Supporting Or Lower-Priority Concepts

- FC-001: Read-only LAN monitoring for viewing, browsing, and export without
  remote writes.
- FC-002: Rich sample fields, user-defined 2D sample-map configs/DSLs, sample
  visualizers, and saved views. These may map parameter row keys or user labels
  to visual regions and color/query parameter snapshots without owning sample
  identity or sample geometry.
- FC-011: Resumable execution checkpoints with a managed runner.
- FC-012: External large asset references for detector files, images, and
  waveforms.
- FC-013: User-facing dataset streams, if internal stream support proves useful
  enough to expose later.
- FC-016: Applying stored parameter or setup state back to devices.

Candidate post-MVP stories and requirements live in
`product/future-stories-and-requirements.md`.

## Deferred Discussion Topics

- Whether sample-map configs should be stored with Sample, Sample Session, or a
  separate visualization record for easier discovery.
- How visualizers declare the parameter snapshot schema they expect.
- How Fricon reports, disables, migrates, or versions visualizers when a
  parameter snapshot schema changes.

## Historical Inputs

Use `dev-docs/v0.2/future-concepts.md` and files under
`dev-docs/v0.2/archive/` as source material only. If they conflict with
accepted `docs-next/` documents, the accepted `docs-next/` document wins.
