# Research Notes

## Status

Draft process note.

## Purpose

`docs/research/` stores lightweight background research and accepted
lessons. It should not become an encyclopedia of external measurement
frameworks.

Use focused research only when a concrete ADR, spec, or design question needs
pressure from existing systems.

## Research Rules

- Prefer official docs, official repos, and project-maintained docs.
- Record source links and review date.
- Distinguish factual observations from Fricon design conclusions.
- Move only durable synthesized lessons into product, domain, architecture, or
  ADR documents.
- Do not let reference-system notes override accepted Fricon ADRs.

## Current Reference-System Passes

Initial review date: 2026-05-06.

Covered:

- QCoDeS and LabRAD Data Vault/Grapher
- Bluesky, Ophyd, Tiled, Databroker, Suitcase
- Labber and ARTIQ/NIST ARTIQ scan framework
- Sacred, MLflow, W&B, DVC, Sumatra, ReproZip, Nextflow, and Snakemake
- EPICS Archiver, Olog, eLabFTW, LabKey, openBIS, and calibration-ledger tools

Accepted synthesis lives in `lessons-for-fricon.md`. The broader source list
and strategic follow-on planning synthesis live in
`strategic-follow-on-future-systems.md`.
Concrete migration pressure from local legacy measurement sample work
directories lives in `legacy-measurement-sample-lessons.md`.
