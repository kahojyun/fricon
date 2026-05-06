# Glossary

## Status

Accepted.

## Public MVP Terms

- Data Library: local Fricon root and catalog for measurements, samples,
  artifacts, and provenance.
- Sample: measured physical object, device, chip, wafer, batch, or specimen.
- Sample Session: cooldown, mount, probing, campaign, or setup period for a
  sample.
- Measurement: data-taking attempt that may produce artifacts and carry
  context, lifecycle, notes, parameters, and code provenance.
- Dataset Artifact: table-shaped artifact produced or consumed by work; owns
  dataset-local facts and semantics.
- Attachment Artifact: small file, image, log, or supporting artifact attached
  to a measurement.
- Parameter Summary: optional light parameter context recorded for a
  measurement; not a full profile or effective-configuration model.
- Run Config Snapshot: optional run-bound snapshot, reference, hash set, or
  summary for selected local files and settings such as parameters, registries,
  wiring references, line/chip info, demod settings, or external runner config.
  It is not a global parameter profile or device inventory.
- Code Provenance Summary: human-readable code context and provenance level.
- Setup Summary: optional passive setup, device, driver, environment, clock, or
  method context; describes, does not control.
- Procedure Summary: optional passive procedure context such as unmanaged
  script, external runner, or declared plan; does not imply managed execution.
- Operator Profile: lightweight local actor label for mutating actions on a
  shared lab computer.
- Event/Audit Record: timeline record for lifecycle, note, correction, system
  action, or actor-labeled mutation.
- Export Bundle: read-only portable package for analysis without importing into
  another data library.
- Export Manifest: read-only package manifest for an export bundle. It records
  package contents, source identity, format version, and integrity metadata.

## Later Or Advanced Terms

- Artifact: durable input or output linked through provenance. Dataset is the
  first concrete type.
- Parameter Snapshot: immutable parameter facts captured by future parameter
  profile or managed-run workflows.
- Parameter Profile: mutable named reference to a useful parameter state.
- Parameter Proposal: reviewed request to update a named parameter profile or
  related setup state from a source run, snapshot, analysis result, or
  calibration result. It carries source evidence and a before/after diff where
  practical.
- Run Manifest: future read model that links available measurement facts such
  as parameter snapshot, code/environment summary, setup/procedure context,
  lifecycle/log events, artifacts, operator, timestamps, calibration evidence,
  review decisions, and provenance coverage. It is not the owner of those
  facts.
- Target Key: user-defined parameter table row key or label that a visualization
  may use to locate a sample-map element. It is not durable sample identity by
  itself.
- Sample Map Config: user-authored JSON or DSL-style description of a 2D sample
  layout and optional mapping from target keys or labels to visual regions. It
  may be stored near a sample for discovery, but its schema compatibility rules
  are a deferred design topic.
- Sample Visualizer: user-authored view that renders parameter snapshot query
  results onto a sample map or other lab-specific visualization. Fricon should
  not assume it understands the physical sample shape.
- Snapshot Query: later product concept for selecting values from a parameter
  snapshot to drive labels, color maps, comparisons, or visualizer state.
- Measurement Code Source: configured upstream source for lab measurement code,
  such as Git/Gitea, package, mirror, or folder. Post-MVP, this should replace
  copied working folders as the normal code provenance story for managed
  measurement, analysis, and calibration work.
- Code Snapshot: immutable resolved code state used by future managed
  execution, analysis evidence, or calibration evidence.
- Generated Sidecar: derived local file, config fragment, waveform, cache, or
  helper artifact produced by code and later consumed by analysis,
  calibration, or replay. It should record source inputs and generator context
  when it affects future interpretation.
- Analysis: later work that consumes artifacts and may produce results,
  reports, or derived datasets.
- Analysis/Fit Attempt: recorded analysis execution or fit attempt with inputs,
  method/code reference, status, diagnostics, quality metrics, failure reason,
  and outputs when available.
- Measurement Outcome: lightweight interpretation or trust decision attached to
  a measurement, such as accepted, questionable, invalidated, or repeat-needed.
  It is not a full electronic lab notebook entry.
- Calibration Record: later record of calibration validity, due/expired state,
  as-found/as-left facts, affected-run windows, and review state.
- Calibration Proposal: reviewed recommendation from calibration evidence that
  may produce parameter or setup changes after approval. It is not a direct
  edit to active configuration.
- Desired Setup State: expected setup or device state derived from parameters,
  routine inputs, and code. It is intent, not proof that hardware changed.
- Observed Device State: readback/status facts for a device or setup, including
  source and freshness where practical.
- Reconciliation Plan: previewed diff from current or observed state to desired
  state, including ordered actions, no-op writes, safe parallel groups,
  settle/readback checks, and abort behavior.
- Apply Execution: audit record for attempting a reconciliation plan, including
  writes, skipped actions, readbacks, failures, and manual overrides.
- Routine Recipe: reviewable description of a repeated compare, run, or analyze
  routine that can be previewed before replay.
- Automation Proposal: previewable plan for a routine or AI-assisted action
  that may read, produce records, or mutate durable state after review.
- Review Decision: approval or rejection record for an automation proposal,
  parameter proposal, or calibration proposal.
- Automation Execution: status record for a reviewed action, including produced
  records and failure handling.

## Avoid As Primary MVP User Terms

- Workspace: v0.1 user-facing term. In the clean-reset product model, keep only
  as historical or internal implementation wording unless an ADR says
  otherwise.
- ActivityRun: internal shared pattern for measurement, analysis, import,
  simulation, and calibration work.
- Stream: internal or advanced substructure for grouped payloads such as
  primary/baseline data. Not an MVP user-facing concept.
- Experiment: informal scientific wording or possible future grouping/template,
  not the first public acquisition record.
