# Architecture Risks

## Status

Draft.

## Risk Register

| ID | Risk | Impact | Mitigation |
| --- | --- | --- | --- |
| RISK-001 | v0.2 grows into too many future concepts before measurement works. | Bloated model and slow delivery. | Keep first slice to data library, measurement, dataset artifacts, optional context, lifecycle, export. |
| RISK-002 | Dataset metadata absorbs measurement/sample/provenance meaning again. | Confusing ownership and hard migrations. | Enforce context map and module boundaries. |
| RISK-003 | Python SDK ergonomics suffer from too much schema ceremony. | Users stay with old logger. | Keep measurement creation short; require scan schema only for plotted data. |
| RISK-004 | Live views slow acquisition writes. | Measurement reliability failure. | Make live views noncritical consumers with bounded queues/coalescing. |
| RISK-005 | Compatibility checks are delayed until after writes. | Corrupted or partial data. | Build client/service/library negotiation early. |
| RISK-006 | Desktop remains dataset-first. | Product fails to become measurement-centered. | Make measurement console the first Desktop workflow. |
| RISK-007 | Reset discards reusable infrastructure unnecessarily. | Slower implementation. | Treat reset as domain reset, not total rewrite. |
| RISK-008 | Old docs continue to guide agents. | Reintroduced v0.1 assumptions. | Route v0.2+ agents through `docs-next/ai/project-context.md`. |
| RISK-009 | Export scope becomes an archive/import system too early. | Delays core measurement loop. | Start with measurement-centered read-only analysis bundle. |
| RISK-010 | AI automation mutates state before audit model exists. | Trust and data integrity risk. | Keep AI read/suggest-only until reviewed mutation ADRs exist. |
