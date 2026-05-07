# v0 Prototype Lessons

## Status

Draft reset postmortem.

## Purpose

Capture the lessons from the v0.1 workspace/dataset-first prototype that must
shape the v0.2+ reset.

## What Worked

- Local-first operation is the right product posture for lab computers.
- Python-led data recording is the right acquisition entry point.
- Arrow-compatible table payloads are a useful base for scientific data.
- Dataset record IDs, append-only facts, chunked payloads, and semantic
  manifests are reusable concepts.
- The Tauri/React desktop stack and Rust/Python build infrastructure are worth
  preserving where they do not force the old model.
- Explicit architecture and maintenance notes helped AI-assisted work avoid
  uncontrolled drift.

## What Failed Or Aged Poorly

- The user-facing model became workspace/dataset-first, but the desired product
  is measurement-history-first.
- Dataset metadata started carrying meaning that belongs to measurements,
  samples, sessions, parameters, code provenance, or lifecycle history.
- Compatibility notes focused on preserving current workspaces before the
  project had real adoption or a stable domain model.
- v0.2 planning, archived proposals, AI process notes, and implementation
  facts accumulated in the same planning space.
- The desktop UI was naturally organized as a dataset explorer instead of a
  measurement console.
- Protocol and service boundaries were shaped around current IPC/gRPC needs,
  not a single future client/service contract shared by Desktop, CLI, and
  Python.
- Future concepts such as parameter snapshots, runner execution, code
  provenance, calibration, and AI automation were discussed before the central
  measurement record was made first-class.

## Concepts That Were Implicit But Need First-Class Treatment

| Concept | Why It Must Be Explicit In v0.2+ |
| --- | --- |
| Data Library | Replaces user-facing workspace as the local root and catalog. |
| Measurement | Owns the data-taking attempt, lifecycle, notes, produced artifacts, and context. |
| Dataset Artifact | Owns table facts and dataset-local semantics, not whole-run meaning. |
| Sample | Represents the measured physical object when known. |
| Sample Session | Represents cooldown, mount, probing, campaign, or setup context. |
| Parameter Snapshot | Captures run facts without requiring a full parameter registry first. |
| Code Provenance Summary | States honestly whether code provenance is unmanaged, user-supplied, or managed. |
| Event/Audit Record | Records lifecycle changes, notes, corrections, and actor labels. |
| Export Bundle | Provides measurement-centered portability without importing into another library. |

## Compatibility Lessons

Pre-v0.2 test workspaces are not product data. Preserving them must not shape
the new model.

After v0.2 records real lab data, the compatibility promise changes:

- preserve recorded data durability
- require explicit migrations or recovery paths
- fail before writes when client/service/library versions are incompatible
- keep v0.x API and protocol shapes free to break when required

## Lessons For Future Agents

- Do not implement from archived proposal files directly.
- Do not treat prototype-facing material as v0.2 product direction.
- Do not put measurement, sample, parameter, code, or lifecycle meaning into
  dataset-local metadata for convenience.
- Do not add future runner, device, calibration, or AI automation surfaces
  before the measurement foundation is coherent.
- If local feature planning introduces a new product term, update
  `docs/product/` and traceability before implementation.
