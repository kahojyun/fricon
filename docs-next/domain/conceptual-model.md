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
    CodeProvenanceSummary
    Event/AuditRecord
    OperatorProfile

  links:
    SampleSession -> Sample
    Measurement -> optional SampleSession
    Measurement -> produces -> DatasetArtifact | AttachmentArtifact
    Measurement -> optional ParameterSnapshot
    Measurement -> optional CodeProvenanceSummary
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
| CodeProvenanceSummary | Provenance level and display summary. | Automatic reproducibility claims for unmanaged code. |
| Event/AuditRecord | Lifecycle events, notes, corrections, actor labels, system actions. | Fine-grained permission enforcement. |

## Reserved Future Model

```text
DataLibrary
  records:
    ActivityRun(kind: measurement | analysis | import | simulation | calibration)
    Artifact(kind: dataset | result | report | log | attachment | parameter_proposal | device_snapshot)
    ParameterProfile
    ParameterSnapshot
    CodeSnapshot
    ScriptRun
    DeviceIdentity
    DeviceSnapshot
    Event/AuditRecord

  links:
    ActivityRun -> consumes -> Artifact | ParameterSnapshot | ActivityRun
    ActivityRun -> produces -> Artifact
    ScriptRun -> optional CodeSnapshot
    Calibration -> coordinates -> Measurement + Analysis + ParameterProposal
```

`ActivityRun` is an internal modeling pattern. Users should see concrete words:
Measurement, Analysis, Import, Simulation, and Calibration.
