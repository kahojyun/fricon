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
- Rerun creates a new linked measurement by default.

## Dataset Artifact

- Dataset facts are append-only while a writer is active.
- Completed dataset facts are immutable.
- Dataset artifacts own dataset-local variable metadata and scan schema.
- Dataset artifacts do not own sample identity, measurement notes, parameter
  history, or code provenance.
- Datasets intended for plotting require explicit scan schema at creation time.
- System-owned fields use a reserved prefix and are hidden from ordinary reads
  unless explicitly requested.

## Provenance And Audit

- Non-managed code provenance must not be presented as reproducible history.
- Mutating actions should record an actor label when practical.
- AI-assisted mutating actions require explicit review and durable audit
  records.
- Corrections preserve history rather than overwriting meaning silently.

## Export

- Measurement-centered export is read-only by default.
- Export bundles carry source library identity, export identity, format version,
  stable record IDs, and checksums where practical.
- Sensitive local paths or provenance details are previewed or opt-in.
