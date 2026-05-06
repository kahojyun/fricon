# Story Map

## Status

Draft.

## Backbone

```text
Install and set up
  -> create/open data library
  -> select optional sample/session context
  -> run Python measurement
  -> record dataset artifacts
  -> inspect live
  -> finish/recover
  -> annotate
  -> reopen in Python
  -> export
```

## v0.2 User Stories

| ID | Persona | Story | Related Capabilities | Acceptance Notes |
| --- | --- | --- | --- | --- |
| US-001 | Experimentalist | Install and launch Fricon on a lab computer. | CAP-001, CAP-002, CAP-013 | Desktop, CLI, service, and Python SDK compatibility is explained; Desktop can start local mode; Python can diagnose service discovery. |
| US-002 | Experimentalist | Create or open one local data library. | CAP-001, CAP-012 | First-run setup asks for location, creates UUID/display name, and does not hide experimental data without consent. |
| US-003 | Experimentalist | Run an exploratory measurement from Python. | CAP-003, CAP-004, CAP-014, CAP-015 | Measurement creation is explicit and short; sample/session is optional; multiple local writers are isolated. |
| US-004 | Experimentalist | Produce dataset artifacts from a measurement. | CAP-005, CAP-006 | Dataset writers share measurement lifecycle; plotted data declares roles and axes. |
| US-005 | Experimentalist | Watch and inspect live data. | CAP-007 | Console shows active and recent measurements; live views cannot block writes. |
| US-006 | Experimentalist | Recover a partial measurement. | CAP-008, CAP-012 | Completed facts remain readable; rerun creates a new linked measurement by default; resume requires explicit checks. |
| US-007 | Experimentalist | Annotate at the right level. | CAP-009, CAP-017 | Favorites/pins and notes attach to measurement/sample/session by default; dataset notes are for local exceptions. |
| US-008 | Analyst | Reopen measurement outputs from Python. | CAP-010 | Snippets use stable IDs and public SDK APIs, not storage paths. |
| US-009 | Analyst | Export a measurement for offline analysis. | CAP-011 | Export includes datasets, selected metadata, checksums, and Python loader information; sensitive provenance is previewed or opt-in. |
| US-010 | Lab Maintainer | Update without corrupting measurement work. | CAP-012, CAP-013 | Updates and migrations check idle state, fail before writes on incompatibility, and create checkpoints where practical. |
| US-011 | Lab Maintainer | Record code provenance honestly. | CAP-014, CAP-020 | Non-managed code is marked unmanaged unless the user supplies a label; future managed snapshots are reserved. |
| US-012 | Experimentalist | Register a sample and sample session when useful. | CAP-004, CAP-019 | Sample/session context is low-friction, visible, and correctable after a run. |
| US-013 | Analyst | Find and open a dataset artifact directly. | CAP-005, CAP-010, CAP-026 | Dataset artifacts are searchable/openable by ID, title, source alias, measurement, time, and context when available. |
| US-014 | Experimentalist | Record passive setup context. | CAP-014, CAP-027 | A measurement can include setup/method labels and optional passive device/environment summaries without device control. |
| US-015 | Experimentalist | Declare common scan shapes quickly. | CAP-006, CAP-028 | Common 1D/2D/N-D scans and traces have helpers; advanced users can provide raw schema. |
| US-016 | Experimentalist | Record passive procedure context. | CAP-014, CAP-029 | A measurement can record unmanaged script, external runner, or declared-plan summary without managed execution. |

## Sequencing

M1 foundation should prove US-001 through US-008 enough to record, inspect, and
reopen a measurement, while preserving first-class dataset discovery. Export,
backup/restore polish, and update flows may start as explicit contracts before
full UX polish. Measurement-centered export should be split into a near-term
SPEC-002 rather than hidden inside the foundation spec.
