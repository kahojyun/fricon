# Context Map

## Status

Draft.

## Bounded Contexts

| Context | Responsibility | Primary Terms | Upstream/Downstream |
| --- | --- | --- | --- |
| Data Library | Local root, identity, catalog, compatibility, backup/restore. | DataLibrary, format version, checkpoint. | Upstream to all data contexts. |
| Measurement | Data-taking records, lifecycle, annotations, produced artifacts, context links. | Measurement, lifecycle, note, marker, event. | Uses Sample, DatasetArtifact, ParameterSnapshot, CodeProvenanceSummary. |
| Dataset Artifact | Table facts, schema, scan semantics, projections, live append state. | DatasetArtifact, variable, role, scan schema, projection. | Produced by Measurement; read by Desktop, Python, Export, Analysis later. |
| Sample Context | Samples, sample sessions, active context, corrections. | Sample, SampleSession, active context. | Used by Measurement and views. |
| Provenance | Code provenance, parameter snapshot, actor/event records. | CodeProvenanceSummary, ParameterSnapshot, Actor, Event. | Linked by Measurement; extended by future runner/calibration. |
| Service/API | Client compatibility, mutations, events, binary payload transfer. | Service API, capability, write session, event stream. | Exposes domain contexts to Desktop, Python SDK, CLI. |
| Desktop Experience | Measurement console, live views, sample/session UX, diagnostics. | Console, live list, detail, detached view. | Downstream of Service/API. |
| Python SDK | Measurement creation, dataset writes, reopen, export reads. | Library handle, Measurement handle, Dataset writer. | Downstream of Service/API. |
| Export | Measurement-centered portable bundles. | ExportBundle, manifest, checksum, offline reader. | Downstream of Measurement and Dataset Artifact. |

## Anti-Corruption Rules

- Dataset metadata may display measurement or sample context but must not own
  it.
- Desktop UI state must not become the durable data backend.
- Python SDK convenience APIs must not bypass service compatibility checks for
  mutating operations.
- Export bundles carry copies for portability; importing or reading an export
  must not imply shared identity with the source data library.
- Future AI, calibration, or automation actions must enter through audited
  domain mutations, not direct storage edits.
