# US-017: Migrate A Data Vault-Style Script

## Status

Draft.

## Story

As a lab user, I want to translate a new Data Vault-style measurement script
into Fricon with minimal conceptual change so that I can stop using the old
logger for new measurements without rewriting the whole experiment stack.

## Acceptance Notes

- The migration path uses Fricon SDK writers, not a Data Vault compatibility
  server.
- Independent/dependent variable declarations map naturally to Fricon scan
  schema helpers or raw schema.
- Labels, units, legends, source aliases, original paths, and old numbered
  titles can be recorded without becoming primary identity.
- User-written import scripts can call generic Fricon APIs for old data later.
- v0.2 does not ship a built-in Data Vault parser or legacy browser.

## Related

CAP-003, CAP-006, CAP-028, CAP-030.
