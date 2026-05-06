# SPEC-002: Measurement Export And Offline Analysis Requirements

## Status

Draft placeholder.

## Purpose

Define the near-term export slice that follows SPEC-001. Export is part of the
v0.2 product promise, but it should not be hidden inside the foundation spec.

## Related Capabilities

CAP-010, CAP-011, CAP-014, CAP-015, CAP-026, CAP-027, CAP-029, CAP-033.

## Requirements To Refine

| ID | Requirement | Acceptance Criteria |
| --- | --- | --- |
| REQ-001 | Export starts from a measurement by default. | Produced dataset artifacts and selected metadata are included. |
| REQ-002 | Exported datasets remain directly readable. | Python can open the export without importing into a data library. |
| REQ-003 | Preserve semantic metadata. | Variable roles, units, scan shape modes, partial semantics, and trace facts travel with the export. |
| REQ-004 | Include source and integrity metadata. | Source data-library identity, export identity, format version, record IDs, and checksums are recorded. |
| REQ-005 | Support common analysis formats where practical. | CSV/Parquet/NetCDF/XArray-oriented paths are evaluated without making lossy formats the source of meaning. |
| REQ-006 | Preview sensitive provenance. | Local paths, source computer labels, detailed environment summaries, and dirty code state are opt-in or previewed. |
| REQ-007 | Preserve passive summaries when selected. | Code provenance, setup summaries, and procedure summaries can travel in the export with privacy preview. |
| REQ-008 | Preserve internal artifact grouping when needed. | Internal streams/subpayloads remain readable without making stream names the primary user-facing concept. |
| REQ-009 | Preserve selected run configuration context. | Run-bound configuration snapshots, references, hashes, and summaries can travel with the export when selected, while sensitive local paths, machine names, IP addresses, and source-folder details are previewed. |

## Non-Goals

- Importing the export into another editable data library as the default path.
- Full offline Desktop viewer polish before the write/reopen loop is proven.
- Legacy Data Vault/Labber/HDF5 compatibility layers.
