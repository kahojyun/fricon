# Glossary

## Status

Draft.

## Terms

| Term | Meaning | Public In v0.2? |
| --- | --- | --- |
| Data Library | The local Fricon root and catalog for measurements, samples, artifacts, and provenance. | Yes |
| Workspace | v0.1 user-facing term for a managed directory. In v0.2+, keep only as historical or internal implementation wording unless an ADR says otherwise. | No |
| Sample | The measured physical object, device, chip, wafer, batch, or specimen. | Yes |
| Sample Session | A cooldown, mount, probing, campaign, or setup period for a sample. | Yes |
| Measurement | A data-taking attempt that may produce artifacts and carry context, lifecycle, notes, parameters, and code provenance. | Yes |
| Dataset Artifact | A table-shaped artifact produced or consumed by work. It owns dataset-local facts and semantics. | Yes |
| Artifact | A durable input or output linked through provenance. Dataset is the first concrete type. | Mostly advanced |
| Attachment Artifact | A small file, image, log, or supporting artifact attached to a measurement. | Yes, light |
| Parameter Snapshot | Immutable parameter facts captured for a measurement. | Yes, minimal |
| Parameter Profile | A mutable named reference to a useful parameter state. | Later |
| Code Provenance Summary | Human-readable summary of code context and provenance level. | Yes |
| Setup Summary | Optional passive summary of setup, device, driver, environment, clock, or method context. It describes context and does not control devices. | Yes, minimal |
| Procedure Summary | Optional passive summary of the measurement procedure, such as unmanaged script, external runner, or declared plan context. It does not imply managed execution. | Yes, minimal |
| Measurement Code Source | Configured upstream source for lab measurement code, such as Git/Gitea, package, mirror, or folder. | Later |
| Code Snapshot | Immutable resolved code state used by future managed execution. | Later |
| ActivityRun | Internal shared pattern for measurement, analysis, import, simulation, and calibration work. | No |
| Analysis | Later work that consumes artifacts and may produce results, reports, or derived datasets. | Later |
| Calibration | Later reviewable workflow that turns measurements/analysis into parameter proposals or approved updates. | Later |
| Operator Profile | Lightweight local actor label for mutating actions on a shared lab computer. | Yes, optional |
| Event/Audit Record | Timeline record for lifecycle, note, correction, system action, or actor-labeled mutation. | Yes |
| Export Bundle | Read-only portable package for analysis without importing into another data library. | Yes |
| Stream | Internal or advanced substructure for grouped payloads such as primary/baseline data. Not a first v0.2 user-facing concept. | No |
| Experiment | Informal scientific wording or possible future grouping/template. Not the first public acquisition record. | No, informal |
