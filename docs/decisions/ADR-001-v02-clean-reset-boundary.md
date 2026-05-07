# ADR-001: v0.2 Clean Reset Boundary

## Status

Accepted.

## Context

Fricon v0.1 is a useful prototype, but its user-facing model is
workspace/dataset-first. The desired v0.2+ product model is a local lab data
library centered on measurements, dataset artifacts, optional sample/session
context, lifecycle history, provenance, and export.

## Decision

v0.2 will be designed as a clean reset. It may break pre-v0.2:

- workspace/storage layout
- public Python workspace/dataset APIs
- IPC/gRPC/protobuf assumptions
- desktop dataset-first navigation
- archive/import/export formats
- setup/update/service compatibility assumptions

Compatibility with old local test workspaces is not a design constraint unless
a later ADR defines a narrow migration/import route.

Useful infrastructure may be reused or adapted, but compatibility must not keep
the wrong user model alive.

Historical code is available on the `archive/v0.1` branch for reference only.
Current documentation must not be steered by obsolete module boundaries.

## Consequences

- The documentation tree becomes the v0.2+ design baseline.
- Early implementation work should prefer replacing current domain boundaries
  in place over building a permanent parallel `fricon-v2` project.
- The first v0.2 implementation must still protect data created by v0.2 once
  real lab data exists.

## Alternatives Considered

- Preserve v0.1 workspace compatibility while adding measurement records. This
  would reduce short-term breakage but preserve the old organizing model.
- Fork a separate v2 project. This would isolate experimentation but duplicate
  build, packaging, testing, and release infrastructure.
- Keep a separate design area while leaving obsolete repository scaffolding
  active. This would keep obsolete implementation cues visible during product
  analysis.

## Revisit Triggers

- Real pre-v0.2 user data appears and needs a migration path.
- A reusable historical component cannot be adapted without carrying discarded
  public semantics forward.
- The project adopts a stable post-v0.x compatibility promise.
