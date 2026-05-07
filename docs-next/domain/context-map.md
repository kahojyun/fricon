# Context Map

## Status

Draft.

## Bounded Contexts

| Context | Responsibility | Primary Terms | Upstream/Downstream |
| --- | --- | --- | --- |
| Data Library | Local root, identity, catalog, compatibility, backup/restore. | DataLibrary, format version, checkpoint. | Upstream to all data contexts. |
| Measurement | Data-taking records, lifecycle, annotations, produced artifacts, context links. | Measurement, lifecycle, note, marker, event. | Uses Sample, DatasetArtifact, ParameterSummary, RunConfigSnapshot, CodeProvenanceSummary. |
| Dataset Artifact | Table facts, schema, scan semantics, projections, live append state. | DatasetArtifact, variable, role, scan schema, projection. | Produced by Measurement; read by Desktop, Python, Export, Analysis later. |
| Sample Context | Samples, sample sessions, active context, corrections. | Sample, SampleSession, active context. | Used by Measurement and views. |
| Provenance | Code provenance, parameter summaries, run-bound configuration snapshots, setup summaries, procedure summaries, actor/event records. | CodeProvenanceSummary, ParameterSummary, RunConfigSnapshot, SetupProvenanceSummary, ProcedureSummary, Actor, Event. | Linked by Measurement; extended by future runner/calibration. |
| Parameter Management | Future named profiles, immutable effective snapshots, diffs, and reviewed proposals. | ParameterProfile, ParameterSnapshot, ParameterProposal, ParameterRef. | Uses Provenance and Analysis evidence; linked by Measurement, Run Manifest, and Calibration. |
| Analysis And Calibration | Future analysis attempts, fit outputs, calibration task chains, calibration records, and calibration proposals. | AnalysisAttempt, CalibrationTaskRun, CalibrationChainRun, CalibrationRecord, CalibrationProposal, GeneratedSidecar. | Consumes DatasetArtifact, ParameterSnapshot, CodeSnapshot; may propose Parameter Management changes after review. |
| Setup/Device Reconciliation | Future desired setup/device state, observed status, reconciliation plans, and apply executions. | DesiredSetupState, ObservedDeviceState, ReconciliationPlan, ApplyExecution. | Consumes ParameterSnapshot, Setup/Device identity, and CodeSnapshot; produces audit events and may bind to Measurement or Routine Replay. |
| Service/API | Client compatibility, mutations, events, binary payload transfer. | Service API, capability, write session, event stream. | Exposes domain contexts to Desktop, Python SDK, CLI. |
| Desktop Experience | Measurement console, live views, sample/session UX, diagnostics. | Console, live list, detail, detached view. | Downstream of Service/API. |
| Python SDK | Measurement creation, dataset writes, reopen, export reads. | Library handle, Measurement handle, Dataset writer. | Downstream of Service/API. |
| Export | Measurement-centered portable bundles. | ExportBundle, manifest, checksum, offline reader. | Downstream of Measurement and Dataset Artifact. |

## Anti-Corruption Rules

- Dataset metadata may display measurement or sample context but must not own
  it.
- Dataset artifacts remain directly searchable and openable; measurement-first
  navigation must not make datasets invisible implementation details.
- Desktop UI state must not become the durable data backend.
- Python SDK convenience APIs must not bypass service compatibility checks for
  mutating operations.
- Export bundles carry copies for portability; importing or reading an export
  must not imply shared identity with the source data library.
- Future AI, calibration, or automation actions must enter through audited
  domain mutations, not direct storage edits.
- Calibration automation must not update named parameter refs directly. It
  should create task evidence, health decisions, chain state, and reviewed
  parameter or calibration proposals for durable promotion.
- Healthy tasks in an approved calibration chain may update a chain-scoped
  calibration working ref for later tasks without requiring review after every
  small step, but the chain must record working-ref revisions, health-gate
  decisions, retries, pause/review reasons, and final promotion outcome.
- Code provenance and parameter snapshots remain separate facts. A run manifest
  may link them, but it must not merge code source, generated config, and
  effective parameter state into one opaque blob.
- Generated sidecars or derived config files that affect analysis,
  calibration, or replay should be artifacts with source inputs and generator
  context.
- Desired setup/device state, observed state, reconciliation plans, and apply
  executions are separate facts. A desired-state routine must not present
  intent as readback evidence.
- Reconciliation may skip, reorder, or parallelize device writes only when the
  relevant device/setup boundary declares that behavior safe.
- Run manifests are read models over available facts. They must not become
  owners of parameter, code, setup, analysis, or artifact records.
- Export manifests describe package contents and integrity. They must not be
  confused with run manifests used for compare, handoff, or investigation.
- Passive setup summaries describe context only. Device control and resource
  ownership belong to later ADR-gated device or runner contexts.
- Passive procedure summaries describe what was intended or invoked. They do
  not imply managed execution, resume support, or runner ownership.
- MVP parameter summaries and run-bound configuration snapshots are lightweight
  context records. They must not be treated as future effective parameter
  snapshots or global parameter-profile bindings.
- AI may summarize or propose, but mutating AI actions must enter through the
  same reviewed proposal and audit boundaries as non-AI automation.
