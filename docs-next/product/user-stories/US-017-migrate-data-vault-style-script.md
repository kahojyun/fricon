# US-017: Migrate A Data Vault-Style Script

## Status

Draft.

## Primary Epic

EPIC-005: Migration ergonomics and future lab scaling.

## Story

As a lab user, I want to translate a new Data Vault-style measurement script
into Fricon with minimal conceptual change so that I can stop using the old
logger for new measurements without rewriting the whole experiment stack.

## Success Criteria

- The migration path uses Fricon SDK writers, not a Data Vault compatibility
  server.
- Independent/dependent variable declarations map naturally to Fricon scan
  schema helpers or raw schema.
- Labels, units, legends, source aliases, original paths, and old numbered
  titles can be recorded without becoming primary identity.
- User-written import scripts can call generic Fricon APIs for old data later.
- v0.2 does not ship a built-in Data Vault parser or legacy browser.

## Not In Scope

- Built-in LabRAD Data Vault parser, compatibility server, or legacy browser.
- Requiring old data to migrate before new Fricon measurements can start.

## Related Capabilities / Specs

CAP-003, CAP-006, CAP-028, CAP-030.
