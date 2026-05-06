# Domain Invariants

## Status

Draft.

## Data Library

- A data library has a generated durable UUID.
- A data library has a storage/format compatibility version.
- Mutating clients must pass compatibility checks before writes.
- The active editable data library is local to the owning machine/service.
- Direct multi-machine shared-folder access to one database-backed data library
  is not supported.

## Measurement

- Measurement identity is stable and independent of display title.
- Measurement display identity prioritizes title/name, start time, and
  sample/session label when available.
- Measurement sample/session links are optional and correctable with history.
- Measurement lifecycle transitions are recorded as events.
- Partial/interrupted measurements remain readable.
- Readable partials are guaranteed before resumable execution is promised.
- Rerun creates a new linked measurement by default.

## Dataset Artifact

- Dataset facts are append-only while a writer is active.
- Completed dataset facts are immutable.
- Dataset artifacts own dataset-local variable metadata and scan schema.
- Dataset artifacts do not own sample identity, measurement notes, parameter
  history, or code provenance.
- Datasets intended for plotting require explicit scan schema at creation time.
- Scan schema supports regular grids, partial grids, irregular/adaptive points,
  repeated points, fixed-shape traces, and variable-length traces.
- Dataset artifacts remain first-class searchable/openable records.
- System-owned fields use a reserved prefix and are hidden from ordinary reads
  unless explicitly requested.

## Provenance And Audit

- Non-managed code provenance must not be presented as reproducible history.
- Run-bound local configuration snapshots record selected references, hashes,
  or summaries only; they must not imply Fricon observed every configuration
  file the script read.
- Passive setup/device/environment summaries must not be presented as device
  control or complete reproducibility records.
- Passive procedure summaries must not be presented as managed execution
  records or resumable plans.
- Managed-run evidence improves provenance coverage but does not guarantee
  scientific reproducibility without parameter, setup/device, environment, and
  calibration coverage.
- Run manifests link available facts and provenance coverage signals; they
  must not silently fill missing context.
- Code provenance, generated sidecars, and effective parameter snapshots are
  separate facts. Linking them for compare or replay must not make any one
  record the owner of the others.
- Calibration-derived parameter changes must preserve source measurements,
  analysis or fit attempts, code context, affected parameter paths,
  before/after diffs, review outcome, and rollback target where practical
  before active refs are updated.
- Calibration automation must not silently mutate active parameter refs,
  setup refs, devices, or generated config that future runs depend on.
- The absence of a detailed confidence-label taxonomy must never be used as a
  reason to allow untracked parameter or calibration mutation.
- Desired setup/device state is intent, not evidence that hardware changed.
  Observed/readback state and apply execution state must remain separate.
- Reconciliation plans must preview no-op writes, required writes,
  dependencies, safe parallel groups, settle/readback checks, timeout behavior,
  and abort behavior before hardware mutation.
- Device writes may be reordered or parallelized only when the affected device
  boundaries explicitly allow it.
- Mutating actions should record an actor label when practical.
- AI-assisted mutating actions require explicit review and durable audit
  records.
- Durable AI-created conclusions should record source/provenance information
  and privacy-scoped inputs where practical.
- Corrections preserve history rather than overwriting meaning silently.

## Export

- Measurement-centered export is read-only by default.
- Export bundles carry source library identity, export identity, format version,
  stable record IDs, and checksums where practical.
- Sensitive local paths or provenance details are previewed or opt-in.
