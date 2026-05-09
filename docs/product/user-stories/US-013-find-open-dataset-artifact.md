# US-013: Find And Open A Dataset Artifact Directly

## Status

Draft pending product-analysis revalidation.

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
- Dataset rows and detail pages support fast stable-ID copy through a
  right-click menu and keyboard shortcut.
- Reopen snippets can target a dataset artifact when that is the useful unit.
- Reader, export, or plot snippets may live in an advanced menu if stable-ID
  copy is the common fast path.

## Not In Scope

- Returning to a dataset-first Desktop home screen.
- Treating internal artifact streams as first-class navigation targets.

## Related Capabilities

CAP-010, CAP-026.
