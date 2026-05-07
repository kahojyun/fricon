# ADR-001: v0.2 Clean Reset Boundary

## Status

Accepted.

## Context

Fricon v0.1 is a useful prototype, but its user-facing model is
workspace/dataset-first. The desired v0.2+ product model is a local lab data
library centered on measurements, dataset artifacts, optional sample/session
context, lifecycle history, provenance, and export.

The old developer-documentation area also mixed current implementation notes,
historical proposal inputs, AI-agent process guidance, and new v0.2 design
direction. That made it too easy for future work to preserve v0.1 assumptions
accidentally.

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

The pre-v0.2 implementation is archived on the `archive/v0.1` branch. The
current branch may remove v0.1 code and implementation scaffolding so product
and domain analysis are not steered by obsolete module boundaries.

## Consequences

- `docs/` becomes the v0.2+ design baseline and the single documentation
  directory.
- Legacy public and developer documentation content may be deleted after useful
  product and domain ideas are merged.
- Pre-v0.2 implementation code and project-specific implementation scaffolding
  may be deleted from the active branch after an archive branch exists.
- Early implementation work should prefer replacing current domain boundaries
  in place over building a permanent parallel `fricon-v2` project.
- The first v0.2 implementation must still protect data created by v0.2 once
  real lab data exists.

## Alternatives Considered

- Preserve v0.1 workspace compatibility while adding measurement records. This
  would reduce short-term breakage but preserve the old organizing model.
- Fork a separate v2 project. This would isolate experimentation but duplicate
  build, packaging, testing, and release infrastructure.
- Keep all planning under the old v0.2 developer-doc area. This would continue
  the current confusion between historical proposals and accepted v0.2+
  baseline.

## Revisit Triggers

- Real pre-v0.2 user data appears and needs a migration path.
- A reusable v0.1 component cannot be adapted without preserving old public
  semantics.
- The project adopts a stable post-v0.x compatibility promise.
