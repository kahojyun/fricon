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

## Later Or Advanced Terms

- Artifact: durable input or output linked through provenance. Dataset is the
  first concrete type.
- Parameter Snapshot: immutable parameter facts captured by future parameter
  profile or managed-run workflows.
- Parameter Profile: mutable named reference to a useful parameter state.
- Measurement Code Source: configured upstream source for lab measurement code,
  such as Git/Gitea, package, mirror, or folder.
- Code Snapshot: immutable resolved code state used by future managed
  execution.
- Analysis: later work that consumes artifacts and may produce results,
  reports, or derived datasets.
- Calibration: later workflow that turns measurements/analysis into parameter
  proposals or approved updates.

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
