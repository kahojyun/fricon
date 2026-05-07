# Conceptual Model

## Status

Draft.

## First v0.2 Model

```text
DataLibrary
  contains records:
    Sample
    SampleSession
    Measurement
    DatasetArtifact
    AttachmentArtifact
    ParameterSummary
    RunConfigSnapshot
    CodeProvenanceSummary
    SetupProvenanceSummary
    ProcedureSummary
    Event/AuditRecord
    OperatorProfile

  links:
    SampleSession -> Sample
    Measurement -> optional SampleSession
    Measurement -> produces -> DatasetArtifact | AttachmentArtifact
    Measurement -> optional ParameterSummary
    Measurement -> optional RunConfigSnapshot
    Measurement -> optional CodeProvenanceSummary
    Measurement -> optional SetupProvenanceSummary
    Measurement -> optional ProcedureSummary
    Event/AuditRecord -> subject record
    Event/AuditRecord -> optional OperatorProfile or service actor
```

## Concept Ownership

| Concept | Owns | Must Not Own |
| --- | --- | --- |
| DataLibrary | Library identity, format version, catalog root, remembered local configuration. | Measurement code source as editable code repository, multi-user organization. |
| Sample | Physical object identity, aliases, custom fields, long-lived notes, lifecycle. | Measurement lifecycle, dataset facts. |
| SampleSession | Cooldown/mount/probing/campaign context for a sample. | New physical sample identity unless the object changed. |
| Measurement | Data-taking intent, lifecycle, context links, produced artifacts, notes, favorites, parameter/code links. | Dataset payload facts, global parameter profile mutation, device communication. |
| DatasetArtifact | Typed table facts, append state, scan schema, variable roles, dataset-local display hints. | Sample identity, measurement notes, code provenance, calibration decisions. |
| AttachmentArtifact | Light measurement files, images, or logs. | Full artifact management or row-linked large binary storage before an ADR. |
| ParameterSummary | Optional light parameter context recorded for a measurement before effective parameter snapshots exist. | Global parameter profiles, immutable effective parameter ownership, override semantics, or calibration promotion. |
| RunConfigSnapshot | Selected local configuration references, copied snapshots, hashes, and source labels bound to a measurement. | Effective parameter ownership, code provenance, setup/device identity, procedure intent, run-manifest assembly, automatic tracing of every file read, or reproducibility claims for unmanaged work. |
| CodeProvenanceSummary | Provenance level and display summary. | Automatic reproducibility claims for unmanaged code. |
| SetupProvenanceSummary | Passive setup, device, driver, environment, method, or clock facts supplied by the user or integration. | Device control, resource locking, readback enforcement, or reproducibility claims. |
| ProcedureSummary | Passive summary of unmanaged script, external runner, or declared plan context. | Managed execution, scan-point checkpointing, task scheduling, or device/resource ownership. |
| Event/AuditRecord | Lifecycle events, notes, corrections, actor labels, system actions. | Fine-grained permission enforcement. |

## Parameter And Configuration Boundaries

MVP configuration capture should not collapse future parameter management into
one generic snapshot object. `product/glossary.md` owns term definitions; this
section owns the concept split.

- ParameterSummary: optional user-supplied or imported context for a
  measurement before profiles, refs, overrides, and proposal workflows exist.
- ParameterSnapshot: future immutable effective parameter facts resolved from
  refs, profiles, and overrides.
- ParameterProfileRef: future mutable named pointer such as `main` or
  `latest-good`.
- CalibrationWorkingRef: future chain-scoped mutable pointer used inside an
  approved calibration chain. Small calibration tasks may update it so later
  tasks consume the latest fitted values without publishing those values as
  durable lab state.
- ParameterProposal: future reviewed change request that may update a named
  parameter ref after approval.
- RunConfigSnapshot: selected local file references, copies, hashes, and source
  labels for lab-local configuration. It does not own effective parameters,
  code provenance, setup/device identity, or procedure intent.
- RunManifest: future read model that links available run facts and states
  provenance coverage. It does not own or duplicate the facts it presents.
- CalibrationProposal: future reviewed recommendation from analysis or
  calibration evidence. It may propose durable parameter/profile changes after
  review; it must not silently edit a named parameter ref.

## Desired State And Reconciliation Boundaries

A later managed setup/device model can be more declarative than today's
imperative loop scripts.

- DesiredSetupState: expected setup or device state derived from parameters,
  routine inputs, and code. It is an intent record, not proof that hardware
  changed.
- ObservedDeviceState: readback, status, freshness, and source facts reported
  by an integration or operator.
- ReconciliationPlan: diff between desired state and observed/current state,
  including no-op writes, ordered dependencies, safe parallel groups,
  settle/readback checks, and abort behavior.
- ApplyExecution: attempted writes, readbacks, skipped actions, failures,
  manual overrides, and divergence from the approved plan.

Detailed reconciliation safety rules belong in `domain/invariants.md` and later
device/setup ADRs.

## Code, Parameter, And Calibration Boundaries

Fricon should keep code, parameter, and calibration facts separate even when a
future run manifest presents them together.

Use Calibration as the unqualified domain term for quantum-experiment
parameter calibration. Use Instrument Calibration for hardware/setup
calibration and Setup/Device Reconciliation for desired-state apply/readback.

- Code provenance explains which source, revision, snapshot, or unmanaged
  summary produced the measurement, analysis, or calibration evidence.
- Parameter snapshots own effective parameter facts. Parameter refs remain
  mutable pointers changed through proposal/audit paths.
- Calibration records describe validity, review state, task health,
  affected-run windows, and evidence. Calibration proposals connect fitted
  sample/control-parameter values to proposed durable parameter changes.
- Generated sidecars or derived config files used by analysis or calibration
  should be artifacts with source inputs and generator context, not invisible
  files that future runs depend on by accident.

Detailed calibration-chain and mutation rules belong in `domain/invariants.md`
and the post-MVP backlog.

## Dataset Artifact Shape

`DatasetArtifact` remains first-class. It is produced by a measurement in the
normal path, but it can also be imported, derived by future analysis, or opened
directly from Python/Desktop.

v0.2 should optimize for scan and trace facts:

- regular grids
- partial grids with missing expected points
- irregular or adaptive point clouds
- repeated points with declared duplicate/display policy
- fixed-shape arrays or traces
- variable-length traces with their own coordinate values and settings

Large detector files, images, and external binary assets are ADR-gated. The
model should reserve artifact/reference hooks without making them the first
v0.2 storage problem.

A dataset artifact is one user-facing artifact. Storage, export, or read APIs
may model internal stream-like groups when needed, but streams should not become
a normal v0.2 user-facing concept.

## Reserved Future Model

```text
DataLibrary
  records:
    ActivityRun(kind: measurement | analysis | import | simulation | calibration)
    Artifact(kind: dataset | result | report | log | attachment | generated_sidecar | parameter_proposal | device_snapshot)
    ParameterProfile
    ParameterSnapshot
    CalibrationWorkingRef
    AnalysisAttempt
    CalibrationRecord
    CalibrationProposal
    CalibrationTaskRun
    CalibrationChainRun
    CodeSnapshot
    ScriptRun
    DeviceIdentity
    DeviceSnapshot
    DesiredSetupState
    ReconciliationPlan
    ApplyExecution
    RunManifest(read model)
    Event/AuditRecord

  links:
    ActivityRun -> consumes -> Artifact | ParameterSnapshot | ActivityRun
    ActivityRun -> produces -> Artifact
    ScriptRun -> optional CodeSnapshot
    AnalysisAttempt -> consumes -> Measurement | Artifact
    CalibrationRecord -> links -> Measurement | AnalysisAttempt | ParameterSnapshot | Artifact
    CalibrationTaskRun -> consumes -> Measurement | AnalysisAttempt | ParameterSnapshot | Artifact
    CalibrationTaskRun -> produces -> fitted values | diagnostics | health decision
    CalibrationTaskRun -> may update -> CalibrationWorkingRef
    CalibrationChainRun -> contains -> CalibrationTaskRun
    CalibrationChainRun -> owns -> CalibrationWorkingRef
    CalibrationProposal -> cites -> CalibrationRecord | CalibrationChainRun | AnalysisAttempt | CodeSnapshot | ParameterSnapshot | Artifact
    CalibrationProposal -> may propose -> ParameterProfile change
    ReconciliationPlan -> compares -> DesiredSetupState | DeviceSnapshot
    ApplyExecution -> executes -> ReconciliationPlan
```

`ActivityRun` is an internal modeling pattern. Users should see concrete words:
Measurement, Analysis, Import, Simulation, and Calibration.

Analysis attempts, calibration records, calibration task runs, calibration
chain runs, and calibration proposals remain separate concepts. Their user
stories and requirements live in
`product/future-stories-and-requirements.md`; their hard anti-corruption rules
live in `domain/invariants.md`.
