# Dataset Feature Requirements (v0.1)

Status: DRAFT (post user adjustments; pending design phase)
Date: 2025-08-30
Scope: Dataset lifecycle, Python API, GUI visualization (line & heatmap), multi-dimensional parameter scanning optimization, real-time partial visualization.

## 1. Glossary

- Dataset: A logical collection of tabular data stored as a single Arrow IPC file plus associated metadata in DB and config in `config.json`.
- Metadata (DB): Core descriptive fields persisted in the database (uuid, name, description, favorite, status, tags, created_at).
- Config (`config.json`): Fricon-internal, per-dataset configuration (index_columns, visualization views, future internal settings). Not stored in DB.
- Index Columns: Columns representing multi-dimensional scan parameters (logical axes), used for plotting & heatmap layout – NOT for DB querying.
- Scan Parameters: User-provided or inferred parameter axes for multi-dimensional experiment / parameter sweep producing one row per parameter combination.
- View: A saved visualization configuration (chart type + role mappings + style) for a dataset.
- Writer Context / DatasetWriter: Python context manager handling row-wise or batch streaming write for initial dataset population (single session only).

## 2. High-Level Goals

1. Provide a simple Python-first workflow for creating, incrementally (streamed) writing once, and analyzing datasets as DataFrames.
2. Optimize for multi-dimensional parameter sweep tasks (frequent one-row-at-a-time writes during a single creation session).
3. Offer GUI quick inspection: metadata and two visualization types (line & heatmap) driven by configurable roles (no raw table preview in MVP).
4. Persist visualization + index column choices separately from core metadata for evolvability.
5. Support real-time (or near real-time) plot updates while a dataset is being written (partial dataset visualization tolerance).

## 3. Out-of-Scope (Phase 1)

- Append after completion (post-write mutations).
- Schema evolution (adding/removing/changing column types after initial write start).
- Full text / fuzzy search across datasets.
- Advanced statistics (distinct counts, histograms) and background async stats jobs.
- Multi-language code snippets (only Python supported).
- Dataset-level indexing for query acceleration.

## 4. Assumptions

1. Single-user local environment; no multi-tenant authorization requirements.
2. Arrow IPC file format retained (no Parquet conversion in MVP).
3. Stats (row/column counts, file size) not displayed in list view for MVP.
4. Aborted writes keep partial on-disk artifacts visible to user for manual cleanup.
5. Standard Python exceptions are sufficient (no bespoke exception hierarchy exposed publicly beyond mapping internal errors).

## 5. Dataset Lifecycle & States

- States: Pending -> Writing -> (Completed | Aborted)
- Aborted remains queryable (status=Aborted) and deletable by user.
- No transition back to Pending; no append after Completed.

## 6. File & Persistence Layout

WHEN a dataset is created THEN the system SHALL create a directory `<workspace>/data/<hh>/<uuid>/` (where `<hh>` = first two hex chars of UUID).
WHEN a dataset write completes successfully THEN the system SHALL persist `dataset.arrow` (Arrow IPC stream/file) and `config.json` (if any config was set during or after write) and shall ensure `metadata` is saved to DB and (optionally) replicated to an on-disk `metadata.json` for portability.
IF a write aborts THEN the system SHALL leave any partially written `dataset.arrow` file (may be truncated) and retain DB status=Aborted.

## 7. Metadata (DB) Fields

THE SYSTEM SHALL store per dataset in DB: uuid, name, description, favorite, status, created_at, tags, (future-safe: reserved columns), without `index_columns`.

## 8. Config (`config.json`) Structure (MVP)

```
{
  "version": 1,
  "index_columns": ["col_a", "col_b"],          // optional
  "views": {
     "main": {
        "chart_type": "line" | "heatmap",
        "roles": { "x": "col_x", "y": ["col_y"], "z": "col_z"?, "color": "col_c"? },
        "style": { "palette": "category10"?, "line_width": 2? }
     }
  },
  "default_view": "main"
}
```

## 9. Index Column Capture & Inference

WHEN user supplies `index_columns` during write initialization THEN the system SHALL store them in `config.json` unchanged.
WHEN user does NOT supply `index_columns` THEN the system SHALL DEFER inference until EITHER (a) the dataset write finalizes OR (b) the first visualization that requires index columns is requested (whichever happens first).
WHEN performing deferred inference THEN the system SHALL scan the written rows (entire file if Completed; current flushed portion if still Writing) to choose the smallest contiguous prefix of columns whose tuple of values is unique across scanned rows (no arbitrary row-count cap; may impose a very high safety cap configurable, default unlimited / large).
IF no contiguous prefix yields global uniqueness THEN the system SHALL select the shortest prefix achieving the highest uniqueness ratio (ties broken by shorter length) and mark inference quality as "partial" (internal flag) while still storing that prefix as `index_columns`.
WHEN the dataset is still Writing and subsequent rows later invalidate previously unique prefix uniqueness THEN the system SHALL NOT auto-adjust `index_columns` (user may override manually).
WHEN user explicitly sets index columns (manual override) THEN the system SHALL overwrite any inferred columns.

## 10. Python API Functional Requirements

Ubiquitous requirements (API surface):
THE SYSTEM SHALL provide a Python module `fricon.datasets` exposing:

1. create_writer(name, description="", tags=None, index_columns=None, scan_params_spec=None) -> DatasetWriter (context manager)
2. open_dataset(uuid|name) -> DatasetHandle (read-only + config/views edits)
3. list_datasets(filters: optional) -> list[DatasetInfo]
4. delete_dataset(uuid)
5. update_dataset(uuid, name=?, description=?, favorite=?)
6. add_tags(uuid, tags:list[str]) / remove_tags(uuid, tags:list[str])
7. generate_snippet(uuid) -> str (Python read snippet)
8. set_index_columns(uuid, columns)
9. save_view(uuid, view_name, view_definition) / delete_view(uuid, view_name) / set_default_view(uuid, view_name)

DatasetWriter (returned by create_writer):
WHEN used as a context manager THEN the system SHALL begin a single write session (status: Writing) and accept row-wise or batch additions.
WHEN `add_row(row_dict)` is called THEN the system SHALL validate schema consistency & scan parameter alignment.
WHEN `add_rows(list_of_row_dicts)` is called THEN the system SHALL process rows sequentially with same validation semantics.
WHEN context exits without error THEN the system SHALL finalize Arrow file and mark status=Completed returning a DatasetHandle.
IF an exception escapes THEN the system SHALL mark status=Aborted and return no handle.
IF any write method is called after finalization THEN the system SHALL raise `RuntimeError`.

DatasetHandle methods:
WHEN `to_pandas(columns=None, limit=None)` is called THEN the system SHALL read Arrow data (partial allowed if status=Writing or Aborted) into a DataFrame (columns subset & limit applied in-memory for MVP).
WHEN modifying config (views, index columns) via handle THEN the system SHALL persist changes to `config.json` and emit ConfigUpdated event.

## 11. Multi-Dimensional Parameter Scanning

WHEN creating a dataset the user MAY provide a `scan_params_spec` (list of parameter column names expected to form a Cartesian product).
IF `scan_params_spec` is provided THEN the system SHALL validate that each added row's parameter subset matches the spec name set (no missing / extraneous parameter columns).
WHEN `scan_params_spec` is provided AND all combinations are observed (optional future enhancement) THEN the system MAY offer completeness feedback (NOT in MVP).
IF `scan_params_spec` is absent THEN inference rules (Section 9) apply; validation is limited to consistent schema.

## 12. Visualization Views

WHEN a view is created or updated via Python or GUI THEN the system SHALL persist it in `config.json` under `views`.
WHEN a dataset has no explicit views THEN the system SHALL create a default `main` view placeholder with empty roles.
WHEN `chart_type` = `heatmap` THEN the system SHALL require roles: x, y, z (value) explicitly (no implicit fallback); optional role `color` MAY be introduced later (NOT MVP if adds complexity).
WHEN `chart_type` = `line` THEN the system SHALL require roles: x and at least one y; additional index columns MAY implicitly differentiate series (color) when user selects multiple values.
WHEN saving a view THEN the system SHALL allow specifying filters (subset/fixed values of index columns) to pre-slice the dataset for rendering (may be minimal / empty in MVP if implementation complexity high).
IF a required role is missing at save time THEN the system SHALL reject the view with validation error.
WHEN a view is set as default THEN the system SHALL update `default_view`.

## 13. GUI Requirements (MVP)

WHEN user opens dataset list page THEN the system SHALL display: name, tags, status, favorite indicator, created_at ordering (default desc).
WHEN user filters by tag(s) THEN the system SHALL show datasets containing ALL selected tags (logical AND).
WHEN user opens a dataset detail THEN the system SHALL display: metadata fields, available views, current default view rendered (no raw table preview in MVP).
WHEN configuring a plot THEN the system SHALL provide UI to select roles (x, y(s), z for heatmap) from available columns and to select fixed values for remaining index columns OR choose multiple values producing multiple line series.
WHEN user edits metadata (name/description/favorite, tags) THEN the change SHALL persist immediately to DB (and reflect in detail view without reload).
WHEN user copies snippet THEN the system SHALL place the Python snippet string into clipboard.
WHEN dataset status is Writing THEN the default view (if valid) SHALL auto-refresh with newly appended data (see Section 18) else provide a manual refresh control as fallback.

## 14. Tag Management

WHEN adding tags THEN the system SHALL create any non-existing tag records and associate them.
WHEN removing tags THEN the system SHALL only remove associations (tag row reuse by others preserved).

## 15. Error Handling (Python Layer)

IF dataset not found THEN the system SHALL raise `KeyError` in Python.
IF schema mismatch during row write THEN the system SHALL raise `ValueError`.
IF write attempted after completion THEN the system SHALL raise `RuntimeError`.
IF IO errors occur during write THEN the system SHALL surface an `OSError` (wrapped original message).

## 16. Code Snippet Generation

WHEN snippet requested THEN the system SHALL generate a minimal reproducible Python example including dataset UUID and `to_pandas()` call.
IF columns parameter is provided by user in GUI snippet dialog THEN the system SHALL include `columns=[...]` argument.
WHEN generating a snippet THEN the system SHALL include a commented placeholder line for workspace path if relevant, e.g. `# workspace = "/path/to/workspace"`.

## 17. Validation & Integrity

WHEN writing the first row THEN the system SHALL lock in the column order & types inferred from Python (mapping to Arrow types) for the session.
IF subsequent row adds produce a type incompatible with established Arrow field type THEN the system SHALL raise a `ValueError` before persisting that row.
WHEN finalizing write THEN the system SHALL flush and close the Arrow writer before updating status=Completed.

## 18. Real-Time Visualization (Writing / Partial Datasets)

WHEN a dataset is in Writing state THEN the system SHALL support real-time visualization updates without waiting for completion.
WHEN new rows are flushed during a write session THEN the system SHALL emit an incremental update event (e.g., `AppEvent::DatasetRowsAppended { uuid, batch_summary }`).
WHEN the GUI receives an appended event AND the active dataset view references that dataset THEN the GUI SHALL request the new data slice (entire dataset or tail-N incremental API — selection deferred to design).
WHEN reading a partially written dataset for plotting THEN the system SHALL tolerate end-of-stream conditions without treating them as corruption.
IF real-time streaming is unavailable (platform limitation) THEN the system SHALL allow manual refresh producing a consistent partial snapshot.
WHEN a dataset ends Writing (Completed or Aborted) THEN the system SHALL send a terminal lifecycle event prompting final refresh.

## 19. Observability

WHEN dataset lifecycle events occur (Created, WritingStarted, Completed, Aborted, Deleted, ConfigUpdated, RowsAppended) THEN the system SHALL broadcast an app event for GUI to refresh.

## 20. Backward & Forward Compatibility

WHEN reading `config.json` lacking new future fields THEN the system SHALL tolerate & use defaults.
WHEN `config.json` has a higher `version` than supported THEN the system SHALL log a warning and ignore unknown fields (best-effort load).

## 21. Deferred / Future Requirements (Not in MVP)

- Append after completion.
- Schema evolution.
- Advanced statistics & background computation.
- Parquet storage option.
- Full-text search across name/description.
- Multi-language snippet generation.
- View version history.
- Parameter scan completeness reporting.
- Auto-adjust of index columns after initial inference.

## 22. Open Questions (None – all Phase 1 decisions captured)

If new questions arise they will be appended here before design phase.

## 23. Traceability Matrix (Draft)

| Goal                          | Requirements Sections |
| ----------------------------- | --------------------- |
| Python creation & analysis    | 10, 11, 16, 17        |
| Multi-dim scan optimization   | 9, 11                 |
| Visualization (line, heatmap) | 8, 12, 18             |
| GUI inspection                | 13, 16, 18            |
| Config separation             | 6, 8, 9               |
| Abort retention               | 5, 6                  |
| Real-time visualization       | 18                    |
