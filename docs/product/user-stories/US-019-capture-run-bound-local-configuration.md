# US-019: Capture Run-Bound Local Configuration

## Status

Draft pending product-analysis revalidation.

## Primary Epic

EPIC-005: Migration ergonomics and future lab scaling.

## Story

As a Measurement Run Operator, I want to bind selected local configuration
files to a measurement so that later analysis can inspect the lab-local state
without reading copied folders by hand.

## Success Criteria

- A measurement can attach run-bound configuration files at creation time or
  during the run.
- Attached files can include `parameters.json`, `registry.json`, wiring sheets,
  line/chip info, demod settings, readout settings, or external runner
  configuration.
- First adoption prioritizes mutable parameter files and instrument
  information because these are the most likely context users will attach to
  routine measurements.
- Fricon preserves and returns attached files in their original user-supplied
  form.
- A plain attachment list is enough for first adoption; role labels such as
  trusted evidence, user-provided context, or opaque attachment can wait for
  stronger product evidence.
- Fricon records enough source identity to distinguish a copied file, a source
  path reference, a content hash, and a user-entered summary.
- Fricon can provide simple text preview or a way to open files in an external
  editor where practical, without parsing the file into Fricon-owned parameter
  semantics.
- Fricon does not automatically mark user-provided setup or wiring context as
  stale, ambiguous, or trusted unless it has explicit evidence for that
  judgment.
- Corrections to configuration context are visible as events.
- Export previews sensitive local paths, machine names, IP addresses, and other
  local setup details before including them.

## Not In Scope

- Automatic tracing of every file a script reads.
- Whole-notebook capture as the normal first-slice provenance mechanism.
- Attachment role taxonomy as a first-slice requirement.
- Parsing or normalizing arbitrary user parameter files.
- Judging freshness or correctness of opaque setup, wiring, or instrument
  context from file contents alone.
- Global parameter profiles, proposal workflows, or calibration promotion.
- Device control, hardware inventory, or claiming unmanaged execution is fully
  reproducible.

## Related Capabilities

CAP-015, CAP-027, CAP-029, CAP-033.
