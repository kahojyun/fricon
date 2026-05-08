# US-001: Install And Launch Fricon

## Status

Draft pending product-analysis revalidation.

## Primary Epic

EPIC-001: Local setup and data-library adoption.

## Story

As a Local Measurement System Maintainer, I want one clear way to install and
launch Fricon on a lab computer so that measurement users can open Fricon
Desktop and use the Python SDK against a local data library without assembling
incompatible pieces by hand.

## Success Criteria

- Fricon Desktop, CLI, Python SDK, and required local runtime compatibility are
  visible as one product set.
- First-run setup asks for the data-library location.
- Python scripts can run headlessly when Desktop is closed.
- Setup guidance fits ordinary lab computers, including Windows machines and
  locked-down or offline environments.
- Diagnostics explain stopped local runtime components, wrong library, locked
  library, old SDK, incompatible components, or migration-required states.
- Local support information can be exported in a redacted, user-approved form
  when a Local Measurement System Maintainer needs to debug setup problems.

## Not In Scope

- Hosted service, multi-user administration, or direct shared-folder database
  editing.
- Enterprise deployment polish before the local loop works.
- Full source-code or Python-environment management during the MVP.

## Related Capabilities

CAP-001, CAP-002, CAP-013.
