# US-013: Find And Open A Dataset Artifact Directly

## Status

Accepted.

## Primary Epic

EPIC-003: Measurement console and inspection.

## Story

As an Experimental Data Analyst, I want to find and open a produced dataset
artifact directly so that measurement-first navigation does not hide the actual
data I need to inspect or analyze.

## Success Criteria

- Dataset artifacts have stable IDs and searchable labels or context.
- Direct-open routes show the artifact with its parent measurement context.
- Table and plot views can start from the artifact, not only from the parent
  measurement.
- Reopen snippets can target a dataset artifact when that is the useful unit.

## Not In Scope

- Returning to a dataset-first Desktop home screen.
- Treating internal artifact streams as first-class navigation targets.

## Related Capabilities

CAP-010, CAP-026.
