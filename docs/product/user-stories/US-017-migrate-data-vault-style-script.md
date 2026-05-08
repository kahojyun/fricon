# US-017: Migrate A Data Vault-Style Script

## Status

Draft pending product-analysis revalidation.

## Primary Epic

EPIC-005: Migration ergonomics and future lab scaling.

## Story

As a Measurement Method Author, I want to translate a new Data Vault-style
measurement script into Fricon with minimal conceptual change so that the lab
can stop using the old logger for new measurements without rewriting the whole
experiment stack.

## Success Criteria

- The migration path uses Fricon SDK writers, not a Data Vault compatibility
  server or LabRAD-dependent helper module.
- Independent/dependent variable declarations map naturally to Fricon scan
  schema helpers or raw schema.
- Common writer calls shaped like Data Vault `prepDataset`, `Dataset.add`, or
  independent/dependent variable setup can be translated without rewriting the
  whole hardware runner.
- Migration documentation covers non-obvious patterns, such as N-D scans,
  VNA-like coarse/fine trace collections, and minimizer-style irregular logs,
  rather than many near-duplicate 1D and 2D examples.
- Labels, units, legends, source aliases, original paths, and old numbered
  titles can be recorded without becoming primary identity.
- Original Data Vault folder, session, title, numeric ID, and file path can be
  retained as legacy aliases while stable Fricon IDs remain the reopen path.
- Run-bound configuration summaries can carry old parameter/registry file
  references that the old script relied on.
- The writer model does not block later user-written import scripts, but old
  data import is not part of the migration path for starting new work.
- The initial adoption slice does not ship a built-in Data Vault parser or
  legacy browser.

## Not In Scope

- Built-in LabRAD Data Vault parser, compatibility server, LabRAD-dependent
  helper module, or legacy browser.
- Requiring old data to migrate before new Fricon measurements can start.

## Related Capabilities

CAP-003, CAP-006, CAP-028, CAP-030.
