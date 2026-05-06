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
    ParameterSnapshot
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
    Measurement -> optional ParameterSnapshot
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
| ParameterSnapshot | Immutable run facts. | Mutable profile management or calibration promotion. |
| RunConfigSnapshot | Selected local configuration references, copied snapshots, hashes, and source labels bound to a measurement. | Effective parameter ownership, code provenance, setup/device identity, procedure intent, run-manifest assembly, automatic tracing of every file read, or reproducibility claims for unmanaged work. |
| CodeProvenanceSummary | Provenance level and display summary. | Automatic reproducibility claims for unmanaged code. |
| SetupProvenanceSummary | Passive setup, device, driver, environment, method, or clock facts supplied by the user or integration. | Device control, resource locking, readback enforcement, or reproducibility claims. |
| ProcedureSummary | Passive summary of unmanaged script, external runner, or declared plan context. | Managed execution, scan-point checkpointing, task scheduling, or device/resource ownership. |
| Event/AuditRecord | Lifecycle events, notes, corrections, actor labels, system actions. | Fine-grained permission enforcement. |

## Parameter And Configuration Boundaries

MVP configuration capture should not collapse future parameter management into
one generic snapshot object.

- Parameter context summary: optional user-supplied or imported context for a
  measurement before profiles, refs, overrides, and proposal workflows exist.
- ParameterSnapshot: future immutable effective parameter facts resolved from
  refs, profiles, and overrides.
- ParameterProfileRef: future mutable named pointer such as `main` or
  `latest-good`.
- ParameterProposal: future reviewed change request that may update a named
  parameter ref after approval.
- RunConfigSnapshot: selected local file references, copies, hashes, and source
  labels for lab-local configuration. It does not own effective parameters,
  code provenance, setup/device identity, or procedure intent.
- RunManifest: future read model that links available run facts and states
  coverage/provenance confidence. It does not own or duplicate the facts it
  presents.

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
    Artifact(kind: dataset | result | report | log | attachment | parameter_proposal | device_snapshot)
    ParameterProfile
    ParameterSnapshot
    AnalysisAttempt
    CalibrationRecord
    CalibrationProposal
    CodeSnapshot
    ScriptRun
    DeviceIdentity
    DeviceSnapshot
    RunManifest(read model)
    Event/AuditRecord

  links:
    ActivityRun -> consumes -> Artifact | ParameterSnapshot | ActivityRun
    ActivityRun -> produces -> Artifact
    ScriptRun -> optional CodeSnapshot
    AnalysisAttempt -> consumes -> Measurement | Artifact
    CalibrationRecord -> links -> Measurement | AnalysisAttempt | ParameterSnapshot
    CalibrationProposal -> may propose -> ParameterProfile change | Setup change
```

`ActivityRun` is an internal modeling pattern. Users should see concrete words:
Measurement, Analysis, Import, Simulation, and Calibration.

Analysis attempts, calibration records, and calibration proposals should remain
separate concepts. Analysis consumes data and produces results or diagnostics.
Calibration records describe validity, review state, and affected-run windows.
Calibration proposals recommend reviewed parameter or setup changes.
