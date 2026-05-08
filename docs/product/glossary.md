# Glossary

## Status

Draft pending terminology revalidation.

## Public Initial Adoption Terms

- Data Library: local Fricon root and catalog for measurements, samples,
  artifacts, and provenance.
- Sample: measured physical object, device, chip, wafer, batch, or specimen.
- Sample Session: cooldown, mount, probing, campaign, or setup period for a
  sample.
- Measurement: data-taking attempt that may produce artifacts and carry
  context, lifecycle, notes, parameters, and code provenance.
- Dataset Artifact: structured artifact produced or consumed by work; owns
  dataset-local facts and semantics such as scan axes, step records, output
  values, arrays, or traces.
- Attachment Artifact: small file, image, log, or supporting artifact attached
  to a measurement.
- Parameter Summary: optional light parameter context recorded for a
  measurement; not a full profile or effective-configuration model.
- Run Config Snapshot: optional run-bound snapshot, reference, hash set, or
  summary for selected local files and settings such as parameters, registries,
  wiring references, line/chip info, demod settings, or external runner config.
  It is not a global parameter profile or device inventory.
- Code Provenance Summary: initial adoption record of what Fricon can honestly
  know about measurement code, such as unmanaged label, optional
  script/notebook path, copied-folder/source-root label, optional Git summary
  where meaningful, or user-supplied explanation. It is not a managed code
  snapshot or approval record.
- Setup Summary: optional passive setup, device, driver, environment, clock, or
  method context; describes, does not control.
- Procedure Summary: optional passive procedure context such as unmanaged
  script, external runner, or declared plan; does not imply managed execution.
- Measurement Code: user-authored Python that creates, runs, analyzes, or helps
  explain a measurement. In the initial adoption slice this usually means an
  ordinary script, notebook cell flow, Data Vault-style translated script, or
  copied lab folder. Fricon records honest provenance for it; it does not
  package, approve, deploy, snapshot, or execute the code unless later
  managed-run features exist.
- Operator Profile: lightweight local actor label for mutating actions on a
  shared lab computer.
- Event/Audit Record: timeline record for lifecycle, note, correction, system
  action, or actor-labeled mutation.
- Export Bundle: read-only portable package for analysis without importing into
  another data library or running the acquisition-time local runtime.
- Export Manifest: read-only package manifest for an export bundle. It records
  package contents, source library identity, export identity, format version,
  stable record IDs, checksums, and integrity metadata.

## Later Or Advanced Terms

- Artifact: durable input or output linked through provenance. Dataset is the
  first concrete type.
- Parameter Snapshot: immutable parameter facts captured by future parameter
  profile or managed-run workflows.
- Parameter Profile: mutable named reference to a useful parameter state.
- Calibration Working Ref: chain-scoped mutable parameter reference updated by
  small calibration tasks so later tasks can consume the latest fitted values.
  It is not a durable published profile such as `latest-good`.
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
  such as a Git/Gitea repository, package, mirror, or maintained local folder.
  In strategic follow-on slices, this should replace copied working folders as
  the normal code provenance story for managed measurement, analysis, and
  calibration work.
- Managed Run Entry Point: importable Python function, module entry point, or
  small SDK-integrated wrapper selected for opt-in Fricon-managed execution. It
  is not a scheduler job, visual workflow, shell-command launcher, or generic
  automation recipe by itself.
- Code Snapshot: immutable resolved code state used by future managed
  execution, analysis evidence, or calibration evidence. It may include source
  revision, selected file hashes, dirty-state summary, runner entry point, and
  environment hints where practical.
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
- Calibration: Fricon's accepted product term for quantum-experiment parameter
  calibration unless qualified otherwise. It means measurement plus analysis or
  fit that estimates better sample, qubit, gate, pulse, readout, or analysis
  parameters. It does not mean device desired-state apply/readback by default.
- Calibration Record: later record of calibration evidence, task health,
  fitted values, affected-run windows, and review state.
- Calibration Task Run: one small calibration step with source measurements,
  fitted values, diagnostics, health decision, and optional update to a
  calibration working ref.
- Calibration Chain Run: ordered bootstrap, daily, or targeted calibration
  sequence composed of task runs, working-ref revisions, dependencies, retries,
  pauses, and final promotion outcome.
- Calibration Proposal: reviewed recommendation from calibration evidence that
  may publish selected chain results to a durable parameter profile/ref after
  approval. It is not a direct edit to active configuration.
- Calibration Promotion: publishing selected calibration working-ref results to
  a durable named parameter profile/ref with source evidence, diff, actor, and
  rollback target where practical.
- Instrument Calibration: calibration of instruments, electronics, device
  chain, timing, or setup infrastructure. Use this qualified term when the
  workflow is about hardware/setup state rather than experiment parameters.
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

## Avoid As Primary Initial Adoption User Terms

- ActivityRun: internal shared pattern for measurement, analysis, import,
  simulation, and calibration work.
- Stream: internal or advanced substructure for grouped payloads such as
  primary/baseline data. Not user-facing initial adoption terminology.
- Experiment: informal scientific wording or possible future grouping/template,
  not the first public acquisition record.
- Channel or Log: useful comparison terms from other measurement tools, but
  not accepted Fricon object names until dataset and plotting terminology is
  revalidated.
